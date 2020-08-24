use rlb::backend::{Backend, BackendError, BackendPool};
use rlb::balancing::RoundRobinBalancing;
use std::sync::atomic::Ordering;

#[test]
fn backend_new_test() {
    let backend = Backend::new(String::from(":5000"), Some(String::from("/health")));
    assert_eq!(backend.alive.load(Ordering::Acquire), false);
    assert_eq!(backend.byte_traffic.load(Ordering::Acquire), 0);
    assert_eq!(backend.health_endpoint.unwrap(), "/health");
}

#[test]
fn backend_pool_len() {
    let mut pool = BackendPool::new();
    assert_eq!(pool.len(), 0);
    pool.push(Backend::new(String::from(":5000"), None));
    assert_eq!(pool.len(), 1);
}

#[test]
fn backend_pool_from_list() {
    let pool = BackendPool::from_backends_list(vec![
        Backend::new(String::from(":5000"), None),
        Backend::new(String::from(":5001"), None),
    ]);
    assert_eq!(pool.len(), 2);
}

#[test]
fn backend_pool_next_backend_round_robin() {
    let pool = BackendPool::from_backends_list(vec![
        Backend::new(String::from(":5000"), None),
        Backend::new(String::from(":5001"), None),
    ]);
    let algo = RoundRobinBalancing::new();
    let index = pool.next_backend(algo);
    assert_eq!(index, Err(BackendError::NoBackendAlive));
    pool[1].alive.store(true, Ordering::Relaxed);
    let index = pool.next_backend(RoundRobinBalancing::new());
    assert_eq!(index, Ok(1));
}
