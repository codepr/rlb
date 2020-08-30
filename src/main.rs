use rlb::backend::{Backend, BackendPool};
use rlb::server::Server;
use rlb::Config;

fn main() {
    let config = Config::from_file("config.yaml").expect("Error reading config.yaml");
    let backends = config
        .backends()
        .iter()
        .map(|b| Backend::new(b.to_string(), None))
        .collect();
    // Just a testing HTTP local backend
    let pool = BackendPool::from_backends_list(backends);
    let server = Server::new("127.0.0.1:6767".to_string(), 8, pool);
    server.run();
    println!("Shutdown");
}
