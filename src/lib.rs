pub mod backend;
pub mod balancing;
pub mod http;
pub mod server;
pub mod threadpool;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub backends: Vec<String>,
    pub timeout: i64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<std::error::Error>> {
        let f = std::fs::File::open(path)?;
        let config: Config = serde_yaml::from_reader(f)?;
        return Ok(config);
    }
}
