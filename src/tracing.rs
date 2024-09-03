use std::io;

use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{Config, Tracer, TracerProvider};
use opentelemetry_sdk::Resource;

pub fn get_otel_tracer() -> Tracer {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317"); // TODO: this should default to env vars

    let trace_config = Config::default().with_resource(Resource::new(vec![
        KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]));

    let provider: TracerProvider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(trace_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracer = provider.tracer("opentelemetry");
    // global::set_tracer_provider(provider.clone()); // TODO: is this necessary?

    tracer
}

pub fn init_tracing() {
    let tracer = get_otel_tracer();
    let telemetry = OpenTelemetryLayer::new(tracer);

    let stdout_layer = fmt::Layer::new()
        .with_target(false)
        .with_ansi(true)
        .with_file(false)
        .with_writer(io::stdout)
        .with_line_number(false);

    let subscriber = Registry::default()
        .with(stdout_layer)
        .with(telemetry)
        .with(EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
}
