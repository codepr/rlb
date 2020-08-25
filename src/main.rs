use rlb::backend::{Backend, BackendPool};
use rlb::server::Server;

fn main() {
    // Just a testing HTTP local backend
    let pool = BackendPool::from_backends_list(vec![Backend::new(
        String::from("127.0.0.1:6090"),
        Some(String::from("/health")),
    )]);
    let server = Server::new("127.0.0.1:6767".to_string(), 8, pool);
    server.run();
    println!("Shutdown");
}
