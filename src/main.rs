use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use isoku;
use isoku::http::handle;

const THREAD_COUNT: usize = 4;

fn main() {
    let mut threads = Vec::with_capacity(THREAD_COUNT);
    let server = TcpListener::bind("0.0.0.0:5001").unwrap();
    let server = Arc::new(server);
    let glob = isoku::Glob::new();
    glob.channel_list.add_channel("#osu".to_string(), "wojexe is ciota".to_string());
    let glob = Arc::new(glob);
    for _ in 0..THREAD_COUNT {
        let server = server.clone();
        let glob = glob.clone();
        threads.push(thread::spawn(move || {
            handle(server, glob);
        }));
    }
    for handle in threads {
        handle.join().unwrap();
    }
    //handle(server, glob);
    println!("Hello world");
}