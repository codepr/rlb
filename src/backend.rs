use crate::balancing::LoadBalancing;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub struct Backend {
    addr: String,
    pub alive: AtomicBool,
    pub byte_traffic: AtomicUsize,
    pub health_endpoint: Option<String>,
}

impl Backend {
    /// Create a new Backend
    ///
    /// The addr is the connection endpoint representing the backend, health_endpoint is an
    /// `Option` representing an optional healthcheck endpoint
    pub fn new(addr: String, health_endpoint: Option<String>) -> Backend {
        Backend {
            addr,
            alive: AtomicBool::new(false),
            byte_traffic: AtomicUsize::new(0),
            health_endpoint,
        }
    }
}

pub struct BackendPool {
    backends: Vec<Backend>,
}

impl BackendPool {
    /// Create a new BackendPool
    pub fn new() -> BackendPool {
        BackendPool {
            backends: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.backends.len()
    }

    pub fn from_backends_list(backends: Vec<Backend>) -> BackendPool {
        BackendPool { backends }
    }

    pub fn push(&mut self, backend: Backend) {
        self.backends.push(backend);
    }

    pub fn next_backend(&self, mut algo: impl LoadBalancing) -> Option<usize> {
        algo.next_backend(&self.backends)
    }
}
