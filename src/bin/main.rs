use rlb::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

/// Healthcheck route /healthcheck raw bytes format
const healthcheck_header: &[u8; 27] = b"GET /healthcheck HTTP/1.1\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6767").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutdown");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    let response = if buffer.starts_with(healthcheck_header) {
        healthcheck()
    } else {
        handle_request()
    };
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn healthcheck<'a>() -> &'a str {
    /// TODO
    return "HTTP/1.1 200 OK\r\n\r\n";
}

fn handle_request<'a>() -> &'a str {
    /// TODO
    return "HTTP/1.1 200 OK\r\n\r\n";
}
