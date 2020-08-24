use rlb::http;
use std::collections::HashMap;

#[test]
fn http_parse_request_test() {
    let request_bytes = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let request = http::parse_request(request_bytes);
    assert_eq!(request.method, http::HttpMethod::Get);
    assert_eq!(request.http_version, http::HttpVersion::V11);
    assert_eq!(request.headers.contains_key("Host"), true);
}

fn http_request_to_string_test() {
    let request_str = "GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let request = http::Request {
        method: http::HttpMethod::Get,
        http_version: http::HttpVersion::V11,
        route: "/hello".to_string(),
        headers: [("Host".to_string(), "localhost".to_string())]
            .iter()
            .cloned()
            .collect(),
        body: None,
    };
    assert_eq!(format!("{}", request), request_str);
}
