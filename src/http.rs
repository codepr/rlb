use std::collections::HashMap;
use std::fmt;

const CRLF: &str = "\r\n\r\n";

#[derive(Debug, PartialEq)]
pub enum HttpError {
    ParsingError,
    InvalidStatusCode,
}

#[derive(Debug, Clone, PartialEq, Copy)]
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

pub struct StatusCode(u16);

impl StatusCode {
    pub fn from_str(&self, str: &String) -> Result<StatusCode, HttpError> {
        let bytes = str.as_bytes();
        if bytes.len() < 3 {
            return Err(HttpError::InvalidStatusCode);
        }

        let a = bytes[0].wrapping_sub(b'0') as u16;
        let b = bytes[1].wrapping_sub(b'0') as u16;
        let c = bytes[2].wrapping_sub(b'0') as u16;

        if a == 0 || a > 5 || b > 9 || c > 9 {
            return Err(HttpError::InvalidStatusCode);
        }

        let status = (a * 100) + (b * 10) + c;
        Ok(StatusCode(status))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get(String),
    Post(String),
    Put(String),
    Delete(String),
    Connect(String),
    Head,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpMethod::Get(r) => write!(f, "GET {}", r),
            HttpMethod::Post(r) => write!(f, "POST {}", r),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Put(r) => write!(f, "PUT {}", r),
            HttpMethod::Delete(r) => write!(f, "DELETE {}", r),
            HttpMethod::Connect(r) => write!(f, "CONNECT {}", r),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpHeader {
    Method(HttpVersion, HttpMethod),
    Status(HttpVersion, String),
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpHeader::Method(v, m) => write!(f, "{} {}", m, v),
            HttpHeader::Status(v, s) => write!(f, "{} {}", s, v),
        }
    }
}

pub struct HttpMessage {
    pub header: HttpHeader,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpMessage {
    pub fn method(&self) -> Option<&HttpMethod> {
        match &self.header {
            HttpHeader::Method(_, m) => Some(m),
            HttpHeader::Status(_, _) => None,
        }
    }
    pub fn http_version(&self) -> Option<&HttpVersion> {
        match &self.header {
            HttpHeader::Method(v, _) => Some(v),
            HttpHeader::Status(v, _) => Some(v),
        }
    }

    pub fn transfer_encoding(&self) -> Option<&String> {
        self.headers.get("Transfer-Encoding")
    }

    pub fn route(&self) -> Option<&String> {
        match self.method() {
            Some(method) => match method {
                HttpMethod::Get(route)
                | HttpMethod::Post(route)
                | HttpMethod::Put(route)
                | HttpMethod::Connect(route)
                | HttpMethod::Delete(route) => Some(route),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for HttpMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut headers_str = String::new();
        for (k, v) in self.headers.iter() {
            headers_str.push_str(&format!("{}: {}\r\n", k, v));
        }
        let body = match &self.body {
            Some(b) => b,
            None => "",
        };
        let repr = format!("{}\r\n{}{}{}", self.header, &headers_str, body, CRLF);
        write!(f, "{}", repr)
    }
}

/// Parse an HTTP message
///
/// Receive a buffer argument representing a bytearray received from an
/// open stream.
///
/// # Panics
///
/// The `parse_header` function will panic in case of missing mandatory fields
/// like HTTP version, a supported valid method
pub fn parse_message(buffer: &[u8]) -> Result<HttpMessage, HttpError> {
    let request_str = String::from_utf8_lossy(&buffer[..]);
    let content: Vec<&str> = request_str.split(CRLF).collect();
    let mut chunk = content[0].split_whitespace();
    let mut first_line = content[0].split_whitespace();
    let (version, route) = if content[0].starts_with("HTTP") {
        (
            if content[0].starts_with("HTTP/1.0") {
                HttpVersion::V10
            } else {
                HttpVersion::V11
            },
            None,
        )
    } else {
        let r = chunk.nth(1).unwrap_or("/").to_string();
        (
            if chunk.next().unwrap().starts_with("HTTP/1.0") {
                HttpVersion::V10
            } else {
                HttpVersion::V11
            },
            Some(r),
        )
    };
    let heading = match first_line.next() {
        Some("GET") => HttpHeader::Method(version, HttpMethod::Get(route.unwrap())),
        Some("POST") => HttpHeader::Method(version, HttpMethod::Post(route.unwrap())),
        Some("PUT") => HttpHeader::Method(version, HttpMethod::Put(route.unwrap())),
        Some("DELETE") => HttpHeader::Method(version, HttpMethod::Delete(route.unwrap())),
        Some("CONNECT") => HttpHeader::Method(version, HttpMethod::Connect(route.unwrap())),
        Some("HEAD") => HttpHeader::Method(version, HttpMethod::Head),
        Some(_) => HttpHeader::Status(version, first_line.next().unwrap().to_string()),
        None => return Err(HttpError::ParsingError),
    };
    let mut headers: HashMap<String, String> = HashMap::new();
    let hdr_content: Vec<&str> = content[0].split("\r\n").collect();
    // Populate headers map, starting from 1 as index to skip the first line which
    // contains just the HTTP method and route
    for i in 1..hdr_content.len() {
        let kv: Vec<&str> = hdr_content[i].split(":").collect();
        headers.insert(kv[0].to_string(), kv[1].trim().to_string());
    }
    let body = content[1].trim_matches(char::from(0)).to_string();
    Ok(HttpMessage {
        header: heading,
        headers,
        body: Some(body),
    })
}
