use rlb::backend::Backend;
use rlb::balancing::{HashingBalancing, LeastTrafficBalancing, LoadBalancing, RoundRobinBalancing};
use rlb::http::{HttpMessage, HttpMethod};
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

#[test]
fn least_traffic_test() {
    let mut rr_algo = LeastTrafficBalancing;
    let mut backends = vec![
        Backend::new(String::from(":5000"), None),
        Backend::new(String::from(":5001"), None),
        Backend::new(String::from(":5002"), None),
        Backend::new(String::from(":5003"), None),
    ];
    backends[0].increase_byte_traffic(45);
    backends[1].increase_byte_traffic(40);
    backends[2].increase_byte_traffic(60);
    backends[3].increase_byte_traffic(70);
    let index = rr_algo.next_backend(&backends);
    assert_eq!(index, None);
    for backend in backends.iter() {
        backend.alive.store(true, Ordering::Relaxed);
    }
    let index = rr_algo.next_backend(&backends).unwrap();
    assert_eq!(index, 1);
}

#[test]
fn hashing_test() {
    let request = HttpMessage::new(
        HttpMethod::Get("/hello".to_string()),
        [("Host".to_string(), "localhost".to_string())]
            .iter()
            .cloned()
            .collect(),
    );
    let mut rr_algo = HashingBalancing::new(&request);
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
    assert_eq!(index, 3);
}
