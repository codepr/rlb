use crate::backend::Backend;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait LoadBalancing {
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize>;
}

pub struct RoundRobinBalancing {
    next_index: AtomicUsize,
}

impl RoundRobinBalancing {
    /// Create a new RoundRobinBalancing algorithm.
    pub fn new() -> RoundRobinBalancing {
        RoundRobinBalancing {
            next_index: AtomicUsize::new(0),
        }
    }
}

impl LoadBalancing for RoundRobinBalancing {
    /// Find an available backend from a vector of `Backend` type objects.
    ///
    /// Returns an `Option<usize>` with the possible index of the next available
    /// backend, if all backends are offline (alive == false) return None.
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize> {
        let index = self.next_index.load(Ordering::Acquire) % backends.len();
        self.next_index.store(index + 1, Ordering::Relaxed);
        if backends[index].alive.load(Ordering::Acquire) {
            Some(index)
        } else {
            None
        }
    }
}

pub struct RandomBalancing;

impl LoadBalancing for RandomBalancing {
    /// Return a randomly choosen backend, the only restriction followed is that
    /// it must be alive and healthy.
    ///
    /// Returns an `Option<usize>` with the possible index of the next available
    /// backend, if all backends are offline (alive == false) return None.
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize> {
        let index = rand::thread_rng().gen_range(0, backends.len());
        if backends[index].alive.load(Ordering::Acquire) {
            Some(index)
        } else {
            None
        }
    }
}

pub struct LeastTrafficBalancing;

impl LoadBalancing for LeastTrafficBalancing {
    /// Find an available backend from a vector of `Backend` type objects based
    /// on their traffic bytes count.
    ///
    /// Returns an `Option<usize>` with the possible index of the next available
    /// backend, if all backends are offline (alive == false) return None.
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize> {
        let index = rand::thread_rng().gen_range(0, backends.len());
        if backends[index].alive.load(Ordering::Acquire) {
            Some(index)
        } else {
            None
        }
    }
}
