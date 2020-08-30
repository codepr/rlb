use rlb::backend::{Backend, BackendPool};
use rlb::balancing::RoundRobinBalancing;
use rlb::server;
use rlb::Config;
use std::error::Error;
use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = Config::from_file("config.yaml").expect("Error reading config.yaml");
    let backends = config
        .backends()
        .iter()
        .map(|b| Backend::new(b.to_string(), None))
        .collect();
    // Just a testing HTTP local backend
    let pool = BackendPool::from_backends_list(backends, RoundRobinBalancing::new());

    // Bind a TCP listener
    let listener = TcpListener::bind("127.0.0.1:6767").await?;
    server::run(listener, pool).await
}
