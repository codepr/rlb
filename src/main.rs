use log::info;
use rlb::backend::{Backend, BackendPool};
use rlb::balancing::get_balancer;
use rlb::server;
use rlb::Config;
use tokio::net::TcpListener;

const CONF_PATH: &str = "config.yaml";

#[tokio::main]
pub async fn main() -> rlb::AsyncResult<()> {
    rlb::init_logging().expect("Can't enable logging");
    let config = Config::from_file(CONF_PATH).expect("Error reading config.yaml");
    let backends = config
        .backends()
        .iter()
        .map(|b| Backend::new(b.to_string(), None))
        .collect();
    if let Ok(balancing_algo) = get_balancer(config.balancing_algorithm()) {
        let pool = BackendPool::from_backends_list(backends, balancing_algo);
        // Bind a TCP listener
        let listener = TcpListener::bind(config.listen_on()).await?;
        info!("Listening on {}", config.listen_on());
        server::run(listener, pool).await?
    }
    Ok(())
}
