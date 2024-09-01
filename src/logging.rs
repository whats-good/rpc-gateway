use std::io;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

pub fn setup_logging() {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            fmt::Layer::new()
                // .without_time()
                .compact()
                .with_ansi(true)
                .with_writer(io::stdout),
        );
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
}
