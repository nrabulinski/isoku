use super::Glob;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpListener;
use std::sync::Arc;

pub mod request;
pub mod response;
pub use request::Request;
pub use response::Response;

pub fn handle(server: Arc<TcpListener>, glob: Arc<Glob>) {
    let glob = glob.as_ref();
    for stream in server.incoming() {
        let mut stream = stream.unwrap();

        trace!("accepting request from {}", stream.peer_addr().unwrap());
        let mut buf = [0_u8; 8192];

        let raw = {
            let len = stream.read(&mut buf).unwrap();
            &buf[..len]
        };
        //println!("{}", String::from_utf8_lossy(&raw));

        // use std::sync::atomic::Ordering;
        // glob.req_count.fetch_add(1, Ordering::Relaxed);

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let size = match req.parse(&raw).unwrap() {
            httparse::Status::Complete(s) => s,
            httparse::Status::Partial => panic!("Partial request"),
        };
        let headers = unsafe {
            let mut hashmap = HashMap::new();
            for httparse::Header { name, value } in req.headers {
                hashmap.insert(name.to_lowercase(), std::str::from_utf8_unchecked(value));
            }
            hashmap
        };
        let body = &raw[size..];
        let req = Request::new(
            req.method.unwrap(),
            body,
            headers,
            req.path.unwrap(),
            stream.peer_addr().unwrap(),
        );
        //println!("This is the {}. request!\nREQUEST FROM: {}\n------------------------------\nRAW\n{}\n------------------------------\nBODY\n{:?}\n------------------------------", glob.req_count.load(Ordering::Relaxed), stream.peer_addr().unwrap(), String::from_utf8_lossy(&raw), String::from_utf8_lossy(&body));

        let response = super::main_handler(&req, glob); //Response::from_raw((b"Content-Type: text/html; charset=utf-8").to_vec(), EASTEREGG.to_vec());
        let response = response.encode();
        trace!("sending response to {}", stream.peer_addr().unwrap());
        debug!(target: "verbose-raw-data", "{}\n{:x?}\n{}", stream.peer_addr().unwrap(), response, String::from_utf8_lossy(&response));
        stream.write_all(&response).unwrap();
        stream.flush().unwrap();
    }
}
