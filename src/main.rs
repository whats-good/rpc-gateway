use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;
use tracing::{info, trace};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use rpc_gateway::{
    http::http_handler,
    logging::setup_logging,
    settings::{RawSettings, Settings},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let settings: Settings = RawSettings::from_config_file("config.toml")
        .unwrap()
        .try_into()
        .unwrap();
    let settings: &'static Settings = settings.leak(); // TODO: maybe we can introduce Arc in the future when we add dynamic settings

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server is listening at: {addr}");

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        trace!("Incoming request");
        tokio::task::spawn(async move {
            // TODO: why not http2?
            http1::Builder::new()
                .serve_connection(io, service_fn(|req| http_handler(req, settings)))
                .await
                .expect("Error serving connection");
        });
    }
}
