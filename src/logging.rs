use std::io;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    EnvFilter,
};

pub fn setup_logging() {
    // let log_file = File::create("kerem.log").expect("could not create log file");
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            fmt::Layer::new()
                // .without_time()
                .compact()
                .with_ansi(true)
                // .with_ansi(false)
                .with_file(true)
                // .with_thread_ids(true) // TODO: is this useful?
                .with_writer(io::stdout)
                // .with_writer(log_file)
                .with_line_number(true),
        );
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
}
