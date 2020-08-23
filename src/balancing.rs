use crate::backend::Backend;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait LoadBalancing {
    fn next_backend(&mut self, backends: &Vec<Backend>) -> Option<usize>;
}

pub struct RoundRobinBalancing {
    next_index: AtomicUsize,
}

impl RoundRobinBalancing {
    pub fn new() -> RoundRobinBalancing {
        RoundRobinBalancing {
            next_index: AtomicUsize::new(0),
        }
    }
}

impl LoadBalancing for RoundRobinBalancing {
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
