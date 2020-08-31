pub mod backend;
pub mod balancing;
pub mod http;
pub mod server;
use serde::Deserialize;
use serde_yaml;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    backends: Vec<String>,
    timeout: i64,
    #[serde(default = "balancing::BalancingAlgorithm::round_robin")]
    balancing: balancing::BalancingAlgorithm,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let f = std::fs::File::open(path)?;
        let config: Config = serde_yaml::from_reader(f)?;
        return Ok(config);
    }

    pub fn backends(&self) -> &Vec<String> {
        &self.backends
    }

    pub fn timeout(&self) -> i64 {
        self.timeout
    }

    pub fn balancing_algorithm(&self) -> &balancing::BalancingAlgorithm {
        &self.balancing
    }
}

pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
