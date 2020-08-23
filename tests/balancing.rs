use rlb::backend::Backend;
use rlb::balancing::{LoadBalancing, RoundRobinBalancing};
use std::sync::atomic::Ordering;

#[test]
fn round_robin_test() {
    let mut rr_algo = RoundRobinBalancing::new();
    let backends = vec![
        Backend::new(String::from(":5000"), None),
        Backend::new(String::from(":5001"), None),
        Backend::new(String::from(":5002"), None),
        Backend::new(String::from(":5003"), None),
    ];
    let index = rr_algo.next_backend(&backends);
    assert_eq!(index, None);
    for backend in backends.iter() {
        backend.alive.store(true, Ordering::Relaxed);
    }
    let index = rr_algo.next_backend(&backends).unwrap();
    assert_eq!(index, 1);
    let index = rr_algo.next_backend(&backends).unwrap();
    assert_eq!(index, 2);
    let index = rr_algo.next_backend(&backends).unwrap();
    assert_eq!(index, 3);
    let index = rr_algo.next_backend(&backends).unwrap();
    assert_eq!(index, 0);
}
