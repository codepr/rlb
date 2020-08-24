use crate::balancing::LoadBalancing;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Debug, PartialEq)]
pub enum BackendError {
    NoBackendAlive,
}

pub struct Backend {
    pub addr: String,
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

    pub fn next_backend(&self, mut algo: impl LoadBalancing) -> Result<usize, BackendError> {
        let mut index = None;
        loop {
            if !self.has_backends_available() {
                break;
            }
            if let Some(i) = algo.next_backend(&self.backends) {
                index = Some(i);
                break;
            }
        }
        match index {
            Some(i) => Ok(i),
            None => Err(BackendError::NoBackendAlive),
        }
    }

    pub fn has_backends_available(&self) -> bool {
        self.backends
            .iter()
            .any(|b| b.alive.load(Ordering::Relaxed) == true)
    }
}

impl Index<usize> for BackendPool {
    type Output = Backend;
    fn index(&self, index: usize) -> &Self::Output {
        &self.backends[index]
    }
}

impl IndexMut<usize> for BackendPool {
    fn index_mut(&mut self, index: usize) -> &mut Backend {
        &mut self.backends[index]
    }
}
