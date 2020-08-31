/// Simple RLB server.
///
/// Provides an async `run` function that instantiate a `Server` and listens for
/// incoming connection, serving each one on a dedicated task.
use crate::backend::{Backend, BackendPool};
use crate::http::{parse_message, HttpMessage, HttpMethod, StatusCode};
use crate::AsyncResult;
use log::error;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::time::{self, delay_for, Duration};

// Fixed read buffer size
const BUFSIZE: usize = 2048;

// Timeout magic value (5s)
const TIMEOUT: u64 = 5000;

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
struct Server {
    listener: TcpListener,
    /// Shared pool handle. Contains the backends and the balancing algorithm chosen
    /// at the start-up of the application. Being an Arc Mutex guarded it's allowed
    /// to be cloned and locked in each task using it.
    pool: Arc<Mutex<BackendPool>>,
}

impl Server {
    /// Create a new Server and run.
    ///
    /// Listen for inbound connections. For each inbound connection, spawn a
    /// task to process that connection.
    ///
    /// # Errors
    ///
    /// Returns `Err` if accepting returns an error. This can happen for a
    /// number reasons that resolve over time. For example, if the underlying
    /// operating system has reached an internal limit for max number of
    /// sockets, accept will fail.
    pub async fn run(&mut self) -> AsyncResult<()> {
        // Let's spawn an healthcheck worker first
        let mut probe_handler = Handler {
            pool: self.pool.clone(),
        };
        tokio::spawn(async move {
            if let Err(e) = probe_handler.probe_backends().await {
                error!("Can't spawn `probe_backends` worker: {}", e);
            }
        });
        // Loop forever on new connections, accept them and pass the handling
        // to a worker
        loop {
            let stream = self.accept().await?;
            // Create the necessary per-connection handler state.
            let handler = Handler {
                pool: self.pool.clone(),
            };
            // Spawn a new task to process the connections.
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    error!("Can't spawn `handle_connection` worker: {}", e);
                };
            });
        }
    }

    /// Accept an inbound connection.
    ///
    /// Errors are handled by backing off and retrying. An exponential backoff
    /// strategy is used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each subsequent
    /// failure doubles the wait time. If accepting fails on the 6th try after
    /// waiting for 64 seconds, then this function returns with an error.
    async fn accept(&mut self) -> AsyncResult<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::delay_for(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}

#[derive(Clone)]
struct Handler {
    /// Shared pool handle. Contains the backends and the balancing algorithm chosen
    /// at the start-up of the application. It's used to call `next_backend` method
    /// and route the requests incoming to the right backend.
    pool: Arc<Mutex<BackendPool>>,
}

impl Handler {
    /// Try to connect to all registered backends in the balance pool.
    ///
    /// The pool is the a shared mutable pointer guarded by a mutex.
    async fn probe_backends(&mut self) -> AsyncResult<()> {
        loop {
            // Add a scope to automatically drop the mutex lock before the sleep,
            // alternatively call `drop(pool)` by hand
            {
                let mut pool = self.pool.lock().await;
                let mut buffer = [0; BUFSIZE];
                // Iterating through all the backends and try to connect to each one, if an error
                // in connection is raised, mark the backend as offline.
                // Also if there's an healthcheck endpoint set for the backend, after a
                // successfull connection try to query the endpoint, if the response is different
                // from a `200 OK` mark the backend as offline.
                for backend in pool.iter_mut() {
                    let backend_addr: SocketAddr = backend
                        .addr
                        .parse()
                        .expect("Unable to parse backend address");
                    match TcpStream::connect(&backend_addr).await {
                        // Connection OK, now check if an health_endpoint is set
                        // and try to query it
                        Ok(mut stream) => match backend.health_endpoint() {
                            Some(h) => {
                                let request = HttpMessage::new(
                                    HttpMethod::Get(h.clone()),
                                    [("Host".to_string(), backend.addr.to_string())]
                                        .iter()
                                        .cloned()
                                        .collect(),
                                );
                                stream.write_all(format!("{}", request).as_bytes()).await?;
                                let n = stream.peek(&mut buffer).await?;
                                stream.read(&mut buffer[..n]).await?;
                                let response = parse_message(&buffer).unwrap();
                                // Health endpoint response inspection
                                if response.status_code() == Some(StatusCode::new(200)) {
                                    backend.set_online()
                                } else {
                                    backend.set_offline()
                                }
                            }
                            None => backend.set_online(),
                        },
                        Err(_) => backend.set_offline(),
                    }
                }
            }
            // Sleep for a defined timeout
            delay_for(Duration::from_millis(TIMEOUT)).await;
        }
    }

    /// Process a single connection.
    ///
    /// First retrieve a valid backend to forward the request to then call `handle_request` method
    /// to forward the content to it and read the response back.
    ///
    /// # Errors
    ///
    /// If no backend are available return an `Err`, this can happen if all backends result
    /// offline. Also return an `Err` in caswe of error reading from the selected backend,
    /// connection can be broken in the mean-time.
    async fn handle_connection(&self, mut stream: TcpStream) -> AsyncResult<()> {
        let mut pool = self.pool.lock().await;
        let mut buffer = [0; BUFSIZE];
        let n = stream.peek(&mut buffer).await?;
        stream.read(&mut buffer[..n]).await?;
        let index = match pool.next_backend() {
            Ok(i) => i,
            Err(e) => {
                stream.shutdown(Shutdown::Both)?;
                return Err(Box::new(e));
            }
        };
        let response = self.handle_request(&buffer, &mut pool[index]).await?;
        stream.write_all(response.as_bytes()).await?;
        Ok(())
    }

    /// Handle request from a client, forward it to a selected backend and response
    /// back to the client, by correcting the `Host` header before forward (not very elegant).
    /// Expects the headers of the response, handling `Chunked` responses with multiple `read`
    /// calls.
    ///
    /// # Errors
    ///
    /// Return an `Err` in case of communication errors with the backend (unable to read data or
    /// write it).
    async fn handle_request(&self, buffer: &[u8], backend: &mut Backend) -> AsyncResult<String> {
        let backend_addr: SocketAddr = backend
            .addr
            .parse()
            .expect("Unable to parse backend address");
        let mut request = parse_message(buffer).unwrap();
        // Update the `Host` header on the request to be forwarded
        *request.headers.get_mut("Host").unwrap() = backend.addr.to_string();
        let mut response_buf = [0; BUFSIZE];
        let mut stream = TcpStream::connect(&backend_addr).await?;
        // Log traffic on the backend
        let bytesout = stream.write(format!("{}", request).as_bytes()).await?;
        backend.increase_byte_traffic(bytesout);
        let mut read_bytes = stream.peek(&mut response_buf).await?;
        stream.read(&mut response_buf[..read_bytes]).await?;
        let response = parse_message(&response_buf).unwrap();
        // Multiple read till the message is completed in CHUNKED mode
        if response.transfer_encoding().unwrap_or(&"".to_string()) == "chunked" {
            while response_buf[read_bytes - 5..read_bytes] != [b'0', b'\r', b'\n', b'\r', b'\n'] {
                read_bytes += stream.peek(&mut response_buf[..read_bytes]).await?;
                stream.read(&mut response_buf[read_bytes..]).await?;
            }
        }
        backend.increase_byte_traffic(read_bytes);
        stream.shutdown(Shutdown::Both)?;
        return Ok(String::from_utf8_lossy(&response_buf[..]).to_string());
    }
}

/// Run a tokio async server, accepts and handle new connections asynchronously.
///
/// Arguments are listener, a bound `TcpListener` and pool a `BackendPool` with type
/// `LoadBalancing`
pub async fn run(listener: TcpListener, pool: BackendPool) -> AsyncResult<()> {
    let mut server = Server {
        listener,
        pool: Arc::new(Mutex::new(pool)),
    };
    server.run().await?;
    Ok(())
}
