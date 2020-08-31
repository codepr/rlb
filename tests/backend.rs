use rlb::backend::{Backend, BackendError, BackendPool};
use rlb::balancing::RoundRobinBalancing;
use std::sync::atomic::Ordering;

#[test]
fn backend_new_test() {
    let backend = Backend::new(String::from(":5000"), Some(String::from("/health")));
    assert_eq!(backend.alive.load(Ordering::Acquire), false);
    assert_eq!(backend.byte_traffic(), 0);
    assert_eq!(backend.health_endpoint(), &Some("/health".to_string()));
}

#[test]
fn backend_pool_len() {
    let mut pool = BackendPool::new(Box::new(RoundRobinBalancing::new()));
    assert_eq!(pool.len(), 0);
    pool.push(Backend::new(String::from(":5000"), None));
    assert_eq!(pool.len(), 1);
}

#[test]
fn backend_pool_from_list() {
    let pool = BackendPool::from_backends_list(
        vec![
            Backend::new(String::from(":5000"), None),
            Backend::new(String::from(":5001"), None),
        ],
        Box::new(RoundRobinBalancing::new()),
    );
    assert_eq!(pool.len(), 2);
}

#[test]
fn backend_pool_next_backend_round_robin() {
    let mut pool = BackendPool::from_backends_list(
        vec![
            Backend::new(String::from(":5000"), None),
            Backend::new(String::from(":5001"), None),
        ],
        Box::new(RoundRobinBalancing::new()),
    );
    let index = pool.next_backend();
    assert_eq!(index, Err(BackendError::NoBackendAlive));
    pool[1].alive.store(true, Ordering::Relaxed);
    let index = pool.next_backend();
    assert_eq!(index, Ok(1));
}
