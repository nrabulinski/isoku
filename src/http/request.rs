use std::collections::HashMap;
use std::net::SocketAddr;

pub enum Method<'a> {
    GET,
    POST,
    OTHER(&'a str)
}

pub struct Request<'a> {
    method: Method<'a>,
    body: &'a [u8],
    headers: HashMap<String, &'a str>,
    path: &'a str,
    addr: SocketAddr
}

impl<'a> Request<'a> {
    pub fn new(method: &'a str, body: &'a [u8], headers: HashMap<String, &'a str>, path: &'a str, addr: SocketAddr) -> Self {
        Request {
            method: match method {
                "GET" => Method::GET,
                "POST" => Method::POST,
                _ => Method::OTHER(method)
            },
            body, headers, path, addr
        }
    }

    pub fn body(&self) -> &'a [u8] {
        self.body
    }

    pub fn body_string(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(self.body)
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn get_header(&self, key: &str) -> Option<&&'a str> {
        self.headers.get(key)
    }

    pub fn ip(&self) -> String {
        self.addr.to_string()
    }
}