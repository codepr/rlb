use std::collections::HashMap;
use std::fmt;

const CRLF: &str = "\r\n\r\n";

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Connect,
    Head,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Connect => write!(f, "CONNECT"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    V10,
    V11,
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpVersion::V10 => write!(f, "HTTP/1.0"),
            HttpVersion::V11 => write!(f, "HTTP/1.1"),
        }
    }
}

pub struct Request {
    pub method: HttpMethod,
    pub http_version: HttpVersion,
    pub route: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut headers_str = String::new();
        for (k, v) in self.headers.iter() {
            headers_str.push_str(&format!("{}: {}\r\n", k, v));
        }
        let body = match &self.body {
            Some(b) => b,
            None => "",
        };
        let repr = format!(
            "{} {} {}\r\n{}{}{}",
            self.method, self.route, self.http_version, &headers_str, body, CRLF
        );
        write!(f, "{}", repr)
    }
}

pub fn parse_request(buffer: &[u8]) -> Request {
    let request_str = String::from_utf8_lossy(&buffer[..]);
    let valid_versions: HashMap<&str, HttpVersion> = [
        ("HTTP/1.0", HttpVersion::V10),
        ("HTTP/1.1", HttpVersion::V11),
    ]
    .iter()
    .cloned()
    .collect();
    let valid_methods: HashMap<&str, HttpMethod> = [
        ("GET", HttpMethod::Get),
        ("POST", HttpMethod::Post),
        ("PUT", HttpMethod::Put),
        ("DELETE", HttpMethod::Delete),
        ("CONNECT", HttpMethod::Connect),
        ("HEAD", HttpMethod::Head),
    ]
    .iter()
    .cloned()
    .collect();
    let lines: Vec<&str> = request_str.split(CRLF).collect();
    let method = valid_methods
        .get(&lines[0].split_whitespace().next().unwrap())
        .unwrap();
    let route = lines[0].split_whitespace().nth(1).unwrap_or("/");
    let version = valid_versions
        .get(&lines[0].split_whitespace().nth(2).unwrap())
        .unwrap();
    let mut headers: HashMap<String, String> = HashMap::new();
    let hdr_lines: Vec<&str> = lines[0].split("\r\n").collect();
    // Populate headers map, starting from 1 as index to skip the first line which
    // contains just the HTTP method and route
    for i in 1..hdr_lines.len() {
        let kv: Vec<&str> = hdr_lines[i].split(":").collect();
        headers.insert(kv[0].to_string(), kv[1].to_string());
    }
    let body = lines[1].trim_matches(char::from(0)).to_string();
    Request {
        method: method.clone(),
        http_version: version.clone(),
        route: route.to_string(),
        headers,
        body: Some(body),
    }
}
