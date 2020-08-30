use crate::backend::{Backend, BackendPool};
use crate::balancing::LoadBalancing;
use crate::http::{parse_message, HttpMessage, HttpMethod, StatusCode};
use crate::AsyncResult;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::time::{self, delay_for, Duration};

// Healthcheck route /health raw bytes format
const BUFSIZE: usize = 2048;

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
#[derive(Debug)]
struct Server<T: LoadBalancing> {
    listener: TcpListener,
    pool: Arc<Mutex<BackendPool<T>>>,
}

impl<T> Server<T>
where
    T: LoadBalancing + Send + Sync + 'static,
{
    /// Create a new Server and run.
    pub async fn run(&mut self) -> AsyncResult<()> {
        let mut probe_handler = Handler {
            pool: self.pool.clone(),
        };
        tokio::spawn(async move {
            if let Err(e) = probe_handler.probe_backends().await {
                println!("Error {}", e);
            }
        });
        loop {
            let handler = Handler {
                pool: self.pool.clone(),
            };
            let stream = self.accept().await?;
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    println!("Error {}", e);
                };
            });
        }
    }

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
struct Handler<T: LoadBalancing> {
    pool: Arc<Mutex<BackendPool<T>>>,
}

impl<T> Handler<T>
where
    T: LoadBalancing + Send + Sync + 'static,
{
    /// Try to connect to all registered backends in the balance pool.
    ///
    /// The pool is the a shared mutable pointer guarded by a mutex
    /// The ms argument represents the of milliseconds to sleep between
    /// an healthcheck session and the subsequent.
    async fn probe_backends(&mut self) -> AsyncResult<()> {
        loop {
            // Add a scope to automatically drop the mutex lock before the sleep,
            // alternatively call `drop(pool)` by hand
            {
                let mut pool = self.pool.lock().await;
                let mut buffer = [0; BUFSIZE];
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
            delay_for(Duration::from_millis(5000)).await;
        }
    }

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

    async fn handle_request(&self, buffer: &[u8], backend: &mut Backend) -> AsyncResult<String> {
        let backend_addr: SocketAddr = backend
            .addr
            .parse()
            .expect("Unable to parse backend address");
        let mut request = parse_message(buffer).unwrap();
        *request.headers.get_mut("Host").unwrap() = backend.addr.to_string();
        let mut response_buf = [0; BUFSIZE];
        let mut stream = TcpStream::connect(&backend_addr).await?;
        let bytesout = stream.write(format!("{}", request).as_bytes()).await?;
        backend.increase_byte_traffic(bytesout);
        let mut read_bytes = stream.peek(&mut response_buf).await?;
        stream.read(&mut response_buf[..read_bytes]).await?;
        let response = parse_message(&response_buf).unwrap();
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
pub async fn run<T: LoadBalancing + Send + Sync + 'static>(
    listener: TcpListener,
    pool: BackendPool<T>,
) -> AsyncResult<()> {
    let mut server = Server {
        listener,
        pool: Arc::new(Mutex::new(pool)),
    };
    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                println!("Failed to accept: {}", err);
            }
        },
    };
    Ok(())
}
