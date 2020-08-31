pub mod backend;
pub mod balancing;
pub mod http;
pub mod server;
use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use serde::Deserialize;
use serde_yaml;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logging() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

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
