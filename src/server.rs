use crate::backend::{Backend, BackendPool};
use crate::balancing::RoundRobinBalancing;
use crate::http::parse_message;
use crate::threadpool::ThreadPool;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::Arc;

// Healthcheck route /health raw bytes format
const HEALTHCHECK_HEADER: &str = "GET /health HTTP/1.1\r\n";
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";

pub struct Server {
    addr: String,
    threadpool: ThreadPool,
    balancepool: Arc<BackendPool>,
}

impl Server {
    pub fn new(addr: String, workers: usize, pool: BackendPool) -> Server {
        Server {
            addr,
            threadpool: ThreadPool::new(workers),
            balancepool: Arc::new(pool),
        }
    }

    pub fn run(&self) {
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

mod handlers {

    use super::*;

    pub fn handle_connection(pool: Arc<BackendPool>, mut stream: TcpStream) {
        let mut buffer = [0; 2048];
        stream.read(&mut buffer).unwrap();
        println!("{}", String::from_utf8_lossy(&buffer[..]));
        let balancing_algo = RoundRobinBalancing::new();
        let index = match pool.next_backend(balancing_algo) {
            Ok(i) => i,
            Err(_) => return,
        };
        let response = if buffer.starts_with(HEALTHCHECK_HEADER.as_bytes()) {
            String::from(healthcheck())
        } else {
            handle_request(&buffer, &pool[index])
        };
        println!("{}", response);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn healthcheck<'a>() -> &'a str {
        /// TODO
        return OK_RESPONSE;
    }

    fn handle_request(buffer: &[u8], backend: &Backend) -> String {
        let mut request = parse_message(buffer).unwrap();
        *request.headers.get_mut("Host").unwrap() = backend.addr.to_string();
        let mut response_buf = [0; 2048];
        let mut stream = TcpStream::connect(backend.addr.to_string()).unwrap();
        stream.write(format!("{}", request).as_bytes()).unwrap();
        stream.flush().unwrap();
        let read_bytes = stream.read(&mut response_buf).unwrap();
        stream.read(&mut response_buf[read_bytes..]).unwrap();
        stream
            .shutdown(Shutdown::Both)
            .expect("Unable to shutdown connection");
        return String::from_utf8_lossy(&response_buf[..]).to_string();
    }
}
