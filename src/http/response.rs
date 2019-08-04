use bytes::BufMut;

pub enum StatusCode {
    OK,
    NotFound
}

pub struct Response {
    headers: Option<Vec<u8>>,
    body: Option<Vec<u8>>,
    status: StatusCode,
    //log: bool
}

impl Response {
    pub fn empty() -> Self {
        Response { headers: None, body: None, status: StatusCode::OK }
    }
    
    pub fn empty_nf() -> Self {
        Response { headers: None, body: None, status: StatusCode::NotFound }
    }

    pub fn from_raw(headers: Vec<u8>, body: Vec<u8>) -> Self {
        Response {
            headers: Some(headers), body: Some(body), status: StatusCode::OK
        }
    }

    pub fn set_status(&mut self, s: StatusCode) {
        self.status = s;
    }
}

impl Response {
    pub fn put_headers(&mut self, headers: &[(&str, &str)]) {
        let mut buf = Vec::new();
        for (name, value) in headers {
            let raw = format!("{}: {}\r\n", name, value);
            buf.put(raw.as_bytes());
        }
        self.headers = Some(buf);
    }

    // pub fn log_it(&mut self) {
    //     self.log = true;
    // }
}

impl Response {
    pub fn encode(self) -> Vec<u8> {
        let mut buf: Vec<u8> = match self.status {
            StatusCode::OK => b"HTTP/1.1 200 OK\r\n".to_vec(),
            StatusCode::NotFound => b"HTTP/1.1 404 Not Found\r\n".to_vec()
        };
        
        if let Some(mut headers) = self.headers {
            buf.append(&mut headers);
        }
        if let Some(mut body) = self.body {
            let contlen = format!("Content-Lenght: {}\r\n", body.len());
            buf.put(contlen.as_bytes());
            buf.reserve(2 + body.len());
            buf.put("\r\n");
            buf.append(&mut body);
        }
        // if self.log {
        //     let d = String::from_utf8_lossy(&buf);
        //     println!("{}\n{:?}\n{:x?}\n----\n", d, d, buf);
        // }
        buf
    }
}

impl From<&[u8]> for Response {
    fn from(buf: &[u8]) -> Self {
        Response {
            headers: None,
            body: Some(buf.to_vec()),
            //log: false,
            status: StatusCode::OK
        }
    }
}