use rlb::backend::{Backend, BackendPool};
use rlb::server::Server;

fn main() {
    let pool = BackendPool::from_backends_list(vec![Backend::new(
        String::from("127.0.0.1:9090"),
        Some(String::from("/health")),
    )]);
    let server = Server::new("127.0.0.1:6767".to_string(), 4, pool);
    server.run();
    println!("Shutdown");
}
