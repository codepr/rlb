/// HTTP parsing.
///
/// Provides a `parse_message` function to parse incoming requests or responses from
/// a stream.
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

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

impl HttpVersion {
    pub fn from_str(s: &str) -> HttpVersion {
        if s.starts_with("HTTP/1.0") {
            HttpVersion::V10
        } else if s.starts_with("HTTP/1.1") {
            HttpVersion::V11
        } else {
            panic!("Unsupported HTTP version")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StatusCode(u16);

impl StatusCode {
    pub fn new(code: u16) -> StatusCode {
        StatusCode(code)
    }

    /// Parse status code from the first 3 bytes of the header string.
    ///
    /// # Errors
    ///
    /// Return an `Err` in case of a header line below 3 bytes length or if the code result non
    /// valid (e.g below 100 or over 599, according to the HTTP status codes)
    pub fn from_str(str: &String) -> Result<StatusCode, HttpError> {
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

#[derive(Debug, Clone, PartialEq, Hash)]
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
    pub fn new(method: HttpMethod, headers: HashMap<String, String>) -> HttpMessage {
        HttpMessage {
            header: HttpHeader::Method(HttpVersion::V11, method),
            headers,
            body: None,
        }
    }

    /// Return the method of the request or `None` if it's an HTTP response
    pub fn method(&self) -> Option<&HttpMethod> {
        match &self.header {
            HttpHeader::Method(_, m) => Some(m),
            HttpHeader::Status(_, _) => None,
        }
    }

    /// Return the HTTP version of the request or `None` if it's an HTTP response
    pub fn http_version(&self) -> Option<&HttpVersion> {
        match &self.header {
            HttpHeader::Method(v, _) => Some(v),
            HttpHeader::Status(v, _) => Some(v),
        }
    }

    /// Return the `Transfer-Encoding` value of the response or `None` if it's an HTTP response or
    /// the value is not found.
    pub fn transfer_encoding(&self) -> Option<&String> {
        self.headers.get("Transfer-Encoding")
    }

    /// Return the route of the request or `None` if it's a response or an unknown request type.
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

    /// Return the status code of the response or `None` if it's a request.
    pub fn status_code(&self) -> Option<StatusCode> {
        match &self.header {
            HttpHeader::Status(_, s) => match StatusCode::from_str(&s) {
                Ok(r) => Some(r),
                Err(_) => None,
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
/// # Errors
///
/// Return an `Err(HttpError::ParsingError)` in case of an error parsing the header of the request,
/// this can happen for example if an unknown method appears on the header line.
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

    // Not really solid but separate version and route based on the start of the header line:
    //
    // - If the first line starts with HTTP it's an HTTP response so the HTTP version is the first
    // token we must extract and no route are provided;
    // - Otherwise the version is generally the third token ot be parsed, following the route one
    let (version, route) = if content[0].starts_with("HTTP") {
        (HttpVersion::from_str(&content[0]), None)
    } else {
        let r = chunk.nth(1).unwrap_or("/").to_string();
        (HttpVersion::from_str(&chunk.next().unwrap()), Some(r))
    };

    // Parse the method (verb of the request)
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
