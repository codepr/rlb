use crate::backend::{Backend, BackendPool};
use crate::balancing::RoundRobinBalancing;
use crate::http::parse_message;
use crate::threadpool::ThreadPool;
use std::io;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time};

// Healthcheck route /health raw bytes format
const HEALTHCHECK_HEADER: &str = "GET /health HTTP/1.1\r\n";
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const BUFSIZE: usize = 2048;

pub struct Server {
    addr: String,
    threadpool: ThreadPool,
    balancepool: Arc<Mutex<BackendPool>>,
}

impl Server {
    /// Create a new Server.
    ///
    /// Arguments required are addr in the format of "addr:port", workers
    /// as the number of threads to spawn for serving requests and pool
    /// as the backend pool to balance the request to.
    pub fn new(addr: String, workers: usize, pool: BackendPool) -> Server {
        Server {
            addr,
            threadpool: ThreadPool::new(workers),
            balancepool: Arc::new(Mutex::new(pool)),
        }
    }

    /// Bind a listener to the specified address, start the healthcheck probe thread and serve all
    /// incoming new connections.
    ///
    /// # Panics
    ///
    /// If the bind fails for some reasons and could not listen on the specified address (ex: an
    /// already used address).
    pub fn run(&self) {
        // Start healthcheck worker
        let pool = self.balancepool.clone();
        self.threadpool.execute(|| {
            probe_backends(pool, 5000);
        });
        let listener = TcpListener::bind(self.addr.to_string()).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let pool = self.balancepool.clone();
            self.threadpool.execute(|| {
                handlers::handle_connection(pool, stream);
            });
        }
    }
}

/// Try to connect to all registered backends in the balance pool.
///
/// The pool is the a shared mutable pointer guarded by a mutex
/// The ms argument represents the of milliseconds to sleep between
/// an healthcheck session and the subsequent.
fn probe_backends(pool: Arc<Mutex<BackendPool>>, ms: u64) {
    // Shadow borrow mutable by locking the mutex, impossible to do otherwise
    loop {
        // Add a scope to automatically drop the mutex lock before the sleep,
        // alternatively call `drop(pool)` by hand
        {
            let mut pool = pool.lock().unwrap();
            for backend in pool.iter_mut() {
                match TcpStream::connect(backend.addr.to_string()) {
                    Ok(_) => backend.set_online(),
                    Err(_) => backend.set_offline(),
                }
            }
        }
        thread::sleep(time::Duration::from_millis(ms));
    }
}

mod handlers {

    use super::*;

    pub fn handle_connection(pool: Arc<Mutex<BackendPool>>, mut stream: TcpStream) {
        // Shadow borrow mutable by locking the mutex, impossible to do otherwise
        let pool = pool.lock().expect("Unable to lock the shared pool object");
        let mut buffer = [0; BUFSIZE];
        stream.read(&mut buffer).expect("Unable to read data");
        let balancing_algo = RoundRobinBalancing::new();
        let index = match pool.next_backend(balancing_algo) {
            Ok(i) => i,
            Err(_) => {
                stream
                    .shutdown(Shutdown::Both)
                    .expect("Unable to shutdown connection");
                return;
            }
        };
        let response = if buffer.starts_with(HEALTHCHECK_HEADER.as_bytes()) {
            String::from(healthcheck())
        } else {
            match handle_request(&buffer, &pool[index]) {
                Ok(r) => r,
                Err(e) => panic!("{}", e), // XXX
            }
        };
        stream
            .write(response.as_bytes())
            .expect("Unable to write data");
        stream
            .flush()
            .expect("Unable to flush stream after initial write");
    }

    fn healthcheck<'a>() -> &'a str {
        /// TODO
        return OK_RESPONSE;
    }

    fn handle_request(buffer: &[u8], backend: &Backend) -> Result<String, io::Error> {
        let mut request = parse_message(buffer).unwrap();
        *request.headers.get_mut("Host").unwrap() = backend.addr.to_string();
        let mut response_buf = [0; BUFSIZE];
        let mut stream = TcpStream::connect(backend.addr.to_string())?;
        stream.write(format!("{}", request).as_bytes())?;
        stream.flush()?;
        let mut read_bytes = stream.read(&mut response_buf)?;
        let response = parse_message(&response_buf).unwrap();
        if response.transfer_encoding().unwrap_or(&"".to_string()) == "chunked" {
            while response_buf[read_bytes - 5..read_bytes] != [b'0', b'\r', b'\n', b'\r', b'\n'] {
                read_bytes += stream.read(&mut response_buf[read_bytes..])?;
            }
        }
        stream
            .shutdown(Shutdown::Both)
            .expect("Unable to shutdown connection");
        return Ok(String::from_utf8_lossy(&response_buf[..]).to_string());
    }
}
