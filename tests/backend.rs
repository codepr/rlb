use rlb::backend::{Backend, BackendPool};
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
