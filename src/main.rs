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
    let base = Dispatch::new();

    let filter = |md: &log::Metadata| md.target() != "verbose-raw-data";

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
        .filter(filter)
        .chain(std::io::stdout());

    let main_file = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}:{}]\n{}\n",
                record.level(),
                record.module_path().unwrap(),
                record.file().unwrap(),
                record.line().unwrap(),
                message
            ))
        }).filter(filter)
        .chain(fern::log_file("log/main.log").unwrap());

    let http_data = Dispatch::new()
        .format(|out, message, _| {
            out.finish(format_args!(
                "{}\n", message
            ))
        })
        .filter(|md| md.target() == "verbose-raw-data")
        .chain(fern::log_file("log/http.log").unwrap());

    base.chain(stdout).chain(http_data).chain(main_file).apply().unwrap();
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