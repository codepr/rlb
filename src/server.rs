use crate::backend::BackendPool;
use crate::balancing::RoundRobinBalancing;
use crate::threadpool::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
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
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
        let response = if buffer.starts_with(HEALTHCHECK_HEADER.as_bytes()) {
            healthcheck()
        } else {
            handle_request()
        };
        let balancing_algo = RoundRobinBalancing::new();
        let index = pool.next_backend(balancing_algo);
        match index {
            Ok(i) => println!("Index: {}", i),
            Err(_) => println!("Index not found"),
        }
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn healthcheck<'a>() -> &'a str {
        /// TODO
        return OK_RESPONSE;
    }

    fn handle_request<'a>() -> &'a str {
        /// TODO
        return OK_RESPONSE;
    }
}
