use crate::backend::Backend;
use crate::http::HttpMessage;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
        // Just find the index of the backend with the min value of `bytes_traffic`
        // field
        let index = backends
            .iter()
            .enumerate()
            .min_by_key(|(_, b)| b.byte_traffic())
            .map(|(i, _)| i)
            .unwrap();
        if backends[index].alive.load(Ordering::Acquire) {
            Some(index)
        } else {
            None
        }
    }
}

pub struct HashingBalancing<'a> {
    request: &'a HttpMessage,
}

impl<'a> HashingBalancing<'a> {
    pub fn new(request: &'a HttpMessage) -> HashingBalancing<'a> {
        HashingBalancing { request }
    }
}

impl<'a> LoadBalancing for HashingBalancing<'a> {
    /// Find an available backend from a vector of `Backend` type objects based
    /// on the request hash computed.
    ///
    /// Returns an `Option<usize>` with the possible index of the next available
    /// backend, if all backends are offline (alive == false) return None.
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize> {
        // Just find the index of the backend with the min value of `bytes_traffic`
        // field
        let mut s = DefaultHasher::new();
        let index = match self.request.method() {
            Some(m) => {
                m.hash(&mut s);
                s.finish() as usize % backends.len()
            }
            None => return None,
        };
        if backends[index].alive.load(Ordering::Acquire) {
            Some(index)
        } else {
            None
        }
    }
}
