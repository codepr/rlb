use crate::balancing::LoadBalancing;
use std::error::Error;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Debug, PartialEq)]
pub enum BackendError {
    NoBackendAlive,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Backend error")
    }
}

impl Error for BackendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct Backend {
    pub addr: String,
    pub alive: AtomicBool,
    byte_traffic: AtomicUsize,
    health_endpoint: Option<String>,
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

    pub fn set_online(&mut self) {
        self.alive.store(true, Ordering::Relaxed);
    }

    pub fn set_offline(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
    }

    pub fn increase_byte_traffic(&mut self, bytes: usize) {
        self.byte_traffic.store(bytes, Ordering::Relaxed);
    }

    pub fn byte_traffic(&self) -> usize {
        self.byte_traffic.load(Ordering::Acquire)
    }

    pub fn health_endpoint(&self) -> &Option<String> {
        &self.health_endpoint
    }
}

pub struct BackendPool {
    backends: Vec<Backend>,
    balancing_algo: Box<dyn LoadBalancing + Send + Sync>,
}

impl BackendPool {
    /// Create a new BackendPool
    pub fn new(balancing_algo: Box<dyn LoadBalancing + Send + Sync>) -> BackendPool {
        BackendPool {
            backends: Vec::new(),
            balancing_algo,
        }
    }

    pub fn len(&self) -> usize {
        self.backends.len()
    }

    pub fn from_backends_list(
        backends: Vec<Backend>,
        balancing_algo: Box<dyn LoadBalancing + Send + Sync>,
    ) -> BackendPool {
        BackendPool {
            backends,
            balancing_algo,
        }
    }

    pub fn push(&mut self, backend: Backend) {
        self.backends.push(backend);
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Backend> {
        self.backends.iter_mut()
    }

    pub fn next_backend(&mut self) -> Result<usize, BackendError> {
        let mut index = None;
        // Loop until an available backend is found, checking at every run that
        // there's at least one alive backend to avoid looping forever
        loop {
            if !self.has_backends_available() {
                break;
            }
            if let Some(i) = self.balancing_algo.next_backend(&self.backends) {
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
