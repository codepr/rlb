use rlb::backend::{Backend, BackendPool};
use rlb::balancing::get_balancer;
use rlb::server;
use rlb::Config;
use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> rlb::AsyncResult<()> {
    let config = Config::from_file("config.yaml").expect("Error reading config.yaml");
    let backends = config
        .backends()
        .iter()
        .map(|b| Backend::new(b.to_string(), None))
        .collect();
    if let Ok(balancing_algo) = get_balancer(config.balancing_algorithm()) {
        // Just a testing HTTP local backend
        let pool = BackendPool::from_backends_list(backends, balancing_algo);

        // Bind a TCP listener
        let listener = TcpListener::bind("127.0.0.1:6767").await?;
        server::run(listener, pool).await?
    }
    Ok(())
}
