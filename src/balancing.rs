use rlb::Backend;
use std::io::Ordering;
use std::sync::atomic::AtomicUsize;

pub trait LoadBalancing<T> {
    fn next_backend(&self, backends: Vec<T>) -> Option<T>;
}

pub struct RoundRobinBalancing<T> {
    next_index: AtomicUsize,
}

impl<T> LoadBalancing<T> for RoundRobinBalancing<T> {
    fn next_backend(&mut self, backends: Vec<T>) -> Option<T> {
        let index = self.next_index.load(Ordering::Acquire) % backends.len();
        self.next_index.store(index + 1, Ordering::Acquire);
        return Some(backends[index]);
    }
}
