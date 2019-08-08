#[macro_use]
extern crate log;
extern crate fern;

use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use isoku;
use isoku::http::handle;

const THREAD_COUNT: usize = 4;

fn log_init() {
    use fern::colors::Color;
    use fern::Dispatch;
    let colors = fern::colors::ColoredLevelConfig::new().trace(Color::BrightCyan).debug(Color::BrightMagenta);
    let mut base = Dispatch::new();

    let stdout = Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}]:{} -- {}",
                colors.color(record.level()),
                record.module_path().unwrap(),
                record.line().unwrap(),
                message
            ))
        }).level(log::LevelFilter::Trace)
        .filter(|md| md.target() != "verbose-raw-data")
        .chain(std::io::stdout());

    base.chain(stdout).apply().unwrap();
}

fn main() {
    log_init();
    let mut threads = Vec::with_capacity(THREAD_COUNT);
    let server = TcpListener::bind("0.0.0.0:5001").unwrap();
    let server = Arc::new(server);
    let glob = isoku::Glob::new();
    glob.channel_list.add_channel("#osu".to_string(), "wojexe is ciota".to_string());
    let glob = Arc::new(glob);
    for i in 0..THREAD_COUNT {
        trace!("Spawning thread no {}", i);
        let server = server.clone();
        let glob = glob.clone();
        threads.push(thread::spawn(move || {
            handle(server, glob);
        }));
    }
    for handle in threads {
        handle.join().unwrap();
    }
}