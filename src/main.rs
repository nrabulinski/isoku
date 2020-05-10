use dotenv::dotenv;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use std::{net::SocketAddr, sync::Arc};
use tracing::{instrument, trace, Level};
use tracing_subscriber::FmtSubscriber;

use isoku::{main_handler, Glob};

const EASTER: &str = "<pre>
                    __        
  __  ______  _____/ /_  ____ 
 / / / / __ \\/ ___/ __ \\/ __ \\
/ /_/ / / / / /__/ / / / /_/ /
\\__,_/_/ /_/\\___/_/ /_/\\____/
world's first osu private server written in Rust
</pre>";

#[instrument(skip(glob))]
async fn handle_request(
    req: Request<Body>,
    remote_addr: SocketAddr,
    glob: Arc<Glob>,
) -> http::Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => main_handler(req, glob).await,
        (&Method::GET, "/") => Response::builder().status(200).body(EASTER.into()),
        (&Method::POST, p) if p.starts_with("/api") => {
            Response::builder().status(200).body("API soon(tm)".into())
        }
        _ => {
            trace!("not found");
            Response::builder().status(404).body("Not found".into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let glob = Arc::new(Glob::new().await);

    let service = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        let glob = glob.clone();
        async move {
            Ok::<_, http::Error>(service_fn(move |req| {
                handle_request(req, remote_addr, glob.clone())
            }))
        }
    });

    Server::bind(&"127.0.0.1:5001".parse().unwrap())
        .serve(service)
        .await?;
    Ok(())
}
