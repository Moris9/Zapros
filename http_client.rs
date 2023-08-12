use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use dns_lookup::lookup_host;
use url::Url;

pub enum HttpMethod {
    Get,
    Post,
    Delete,
}

pub struct HttpClient;

#[derive(Debug)]
pub enum HttpRequestError {
    InvalidUrl(String),
    ConnectionError(String),
    SerializationError(serde_json::Error),
}

impl Display for HttpRequestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            HttpRequestError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            HttpRequestError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            HttpRequestError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for HttpRequestError {}

pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub json_body: String,
    pub duration: Duration,
    pub headers: HashMap<String, String>,
}

impl HttpClient {
    fn get_status_text(status_code: u16) -> &'static str {
        match status_code {
            100 => "Continue",
            101 => "Switching Protocols",
            102 => "Processing",
            103 => "Early Hints",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            207 => "Multi-Status",
            208 => "Already Reported",
            226 => "IM Used",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            418 => "I'm a teapot",
            421 => "Misdirected Request",
            422 => "Unprocessable Content",
            423 => "Locked",
            424 => "Failed Dependency",
            425 => "Too Early",
            426 => "Upgrade Required",
            428 => "Precondition Required",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            451 => "Unavailable For Legal Reasons",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",
            506 => "Variant Also Negotiates",
            507 => "Insufficient Storage",
            508 => "Loop Detected",
            510 => "Not Extended",
            511 => "Network Authentication Required",
            _ => "Unknown",
        }
    }

    pub fn request(method: HttpMethod, url: &str, json_body: Option<&serde_json::Value>) -> Result<Option<HttpResponse>, HttpRequestError> {
        let start_time = std::time::Instant::now();

        let parsed_url = Url::parse(url).map_err(|err| HttpRequestError::InvalidUrl(err.to_string()))?;
        let host = parsed_url.host_str().ok_or(HttpRequestError::InvalidUrl("Missing host".to_string()))?;
        let path = parsed_url.path();

        let ip_address = match TcpStream::connect((host, 80)) {
            Ok(_) => host.to_string(),
            Err(_) => match lookup_host(host) {
                Ok(ips) => ips[0].to_string(),
                Err(_) => return Ok(None),
            },
        };

        let server_address = format!("{}:80", ip_address);
        let mut stream = TcpStream::connect(&server_address).map_err(|err| HttpRequestError::ConnectionError(err.to_string()))?;

        let method_str = match method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Delete => "DELETE",
        };

        let mut request = format!(
            "{} {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: Rust-HTTP-Client\r\n\
             Connection: close\r\n\
             \r\n",
            method_str, path, host
        );

        if let Some(body) = json_body {
            let serialized_body = serde_json::to_string(body)
                .map_err(|err| HttpRequestError::SerializationError(err))?;
            request.push_str(&format!("{}\r\n", serialized_body));
        }

        stream.write_all(request.as_bytes()).map_err(|err| HttpRequestError::ConnectionError(err.to_string()))?;

        let mut response = String::new();
        stream.read_to_string(&mut response).map_err(|err| HttpRequestError::ConnectionError(err.to_string()))?;

        let status_line = response.lines().next().unwrap_or("");
        let status_code = status_line.split_whitespace().nth(1)
            .and_then(|code| code.parse::<u16>().ok())
            .unwrap_or(0);

        let status_text = Self::get_status_text(status_code).to_string();

        let headers: HashMap<String, String> = response.lines()
            .skip(1)
            .take_while(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (line.to_string(), "".to_string())
                }
            })
            .collect();

        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|pos| pos + 1).unwrap_or(response.len());
        let json_body = response[json_start..json_end].to_string();

        let duration = start_time.elapsed();

        Ok(Some(HttpResponse { status_code, status_text, json_body, headers, duration }))
    }
}
