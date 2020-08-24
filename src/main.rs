use rlb::backend::{Backend, BackendPool};
use rlb::server::Server;
use std::sync::atomic::Ordering;

fn main() {
    let pool = BackendPool::from_backends_list(vec![Backend::new(
        String::from("127.0.0.1:6090"),
        Some(String::from("/health")),
    )]);
    pool[0].alive.store(true, Ordering::Relaxed);
    let server = Server::new("127.0.0.1:6767".to_string(), 4, pool);
    server.run();
    println!("Shutdown");
}
