use rlb::backend::{Backend, BackendPool};
use rlb::balancing::RoundRobinBalancing;
use rlb::threadpool::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

// Healthcheck route /health raw bytes format
const HEALTHCHECK_HEADER: &str = "GET /health HTTP/1.1\r\n";
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6767").unwrap();
    let threadpool = ThreadPool::new(4);
    let balancepool = Arc::new(BackendPool::from_backends_list(vec![Backend::new(
        String::from("127.0.0.1:9090"),
        Some(String::from("/health")),
    )]));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let pool = balancepool.clone();
        threadpool.execute(|| {
            handle_connection(pool, stream);
        });
    }
    println!("Shutdown");
}

fn handle_connection(pool: Arc<BackendPool>, mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    let response = if buffer.starts_with(HEALTHCHECK_HEADER.as_bytes()) {
        healthcheck()
    } else {
        handle_request()
    };
    let backend_index = pool.next_backend(RoundRobinBalancing::new()).unwrap();
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
