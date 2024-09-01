use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::{io, net::SocketAddr, process::exit, time::Duration};
use tokio::{
    net::TcpListener,
    signal::unix::{signal, Signal, SignalKind},
    time::sleep,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, trace, warn};

use rpc_gateway::{
    http::http_handler,
    logging::setup_logging,
    settings::{RawSettings, Settings},
};

// TODO: handle the cases where the client closes the socket unexpectedly. don't panic when that happens.
// TODO: where should i put the cancellation token? inside the loop, around the loop? around the function?
async fn start_server_loop(settings: &'static Settings, cancellation_token: CancellationToken) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server is listening at: {addr}");
    let listener = TcpListener::bind(addr)
        .await
        .expect("Could not attach to the port.");
    let tracker = TaskTracker::new();
    let server = async {
        loop {
            let stream = match listener.accept().await {
                io::Result::Ok((stream, _)) => stream,
                Err(err) => {
                    error!("Could not accept new connection. {}", err);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            trace!("Incoming request");

            tracker.spawn(async move {
                http1::Builder::new()
                    .serve_connection(io, service_fn(|req| http_handler(req, settings)))
                    .await
                    .expect("Error serving connection");
            });
        }
    };

    tokio::select! {
        biased; // prioritize the shutdown signals

        _ = cancellation_token.cancelled() => {
            tracker.close();
            tracker.wait().await;
            info!("Responded to all ongoing http requests during shutdown.")
        }
        _ = server => {}
    }
}

struct ShutdownSignals {
    sigterm: Signal,
    sigint: Signal,
    cancellation_token: CancellationToken,
}

impl ShutdownSignals {
    fn new(cancellation_token: CancellationToken) -> io::Result<Self> {
        let sigterm = signal(SignalKind::terminate())?;
        let sigint = signal(SignalKind::interrupt())?;
        io::Result::Ok(ShutdownSignals {
            sigterm,
            sigint,
            cancellation_token,
        })
    }

    async fn wait(mut self) {
        tokio::select! {
            _ = self.sigterm.recv() => println!("Received SIGTERM"),
            _ = self.sigint.recv() => println!("Received SIGINT"),
        };

        self.cancellation_token.cancel();
    }

    pub fn spawn(cancellation_token: CancellationToken) {
        match ShutdownSignals::new(cancellation_token) {
            io::Result::Ok(signals) => {
                tokio::spawn(signals.wait());
            }
            io::Result::Err(e) => {
                warn!("Could not attach to SIGINT and SIGTERM signals. Starting the server anyway, but graceful shutdowns will not work as expected. {}", e);
            }
        }
    }
}

fn get_settings() -> &'static Settings {
    let settings: Settings = RawSettings::from_config_file("config.toml")
        .unwrap()
        .try_into()
        .unwrap();
    settings.leak()
}

const GRACEFUL_SHUTDOWN_TIMEOUT: u64 = 5_000;

async fn graceful_shutdown(server_task_tracker: TaskTracker) -> i32 {
    warn!(
        "Attempting graceful shutdown. Timeout is set to: {} ms.",
        GRACEFUL_SHUTDOWN_TIMEOUT
    );

    let code = tokio::select! {
        _ = sleep(Duration::from_millis(GRACEFUL_SHUTDOWN_TIMEOUT)) => {
            error!("Could not gracefully shutdown the server.");
            1
        },
        _ = server_task_tracker.wait() => {
            info!("Graceful shutdown completed successfully.");
            0
        }
    };
    code
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = get_settings();
    setup_logging();

    let token = CancellationToken::new();

    ShutdownSignals::spawn(token.clone());

    let server_task_tracker = TaskTracker::new();
    server_task_tracker.spawn(start_server_loop(settings, token.clone()));
    server_task_tracker.close();

    tokio::select! {
        biased; // prioritize the shutdown signals

        _ = token.cancelled() => {}
        _ = server_task_tracker.wait() => {}
    };

    let code = graceful_shutdown(server_task_tracker).await;

    exit(code);
}
