use rlb::http;

#[test]
fn http_parse_message_test() {
    let request_bytes = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let message = http::parse_message(request_bytes).unwrap();
    assert_eq!(
        message.method(),
        Some(&http::HttpMethod::Get("/hello".to_string()))
    );
    assert_eq!(message.http_version(), Some(&http::HttpVersion::V11));
    assert_eq!(message.headers.contains_key("Host"), true);
    assert_eq!(message.route(), Some(&"/hello".to_string()));
}

#[test]
fn http_request_to_string_test() {
    let request_str = "GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n\r\n";
    let message = http::HttpMessage {
        header: http::HttpHeader::Method(
            http::HttpVersion::V11,
            http::HttpMethod::Get("/hello".to_string()),
        ),
        headers: [("Host".to_string(), "localhost".to_string())]
            .iter()
            .cloned()
            .collect(),

        body: None,
    };
    assert_eq!(format!("{}", message), request_str);
}
