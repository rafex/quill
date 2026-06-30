//! Centralized observability initialization for the quill platform.
//!
//! Each service calls [`init`] at startup. The subscriber wires together:
//! - A human-readable fmt layer controlled by `RUST_LOG` (default: `info`)
//! - An OpenTelemetry layer bridging `tracing` spans/events to OTel
//!
//! **Today**: the OTel provider has no exporter attached — spans are created
//! and propagated through the pipeline but discarded at the end. Zero network
//! overhead, but all the instrumentation points are already in place.
//!
//! **To plug OTLP**: see the commented block in [`init`]. The change is
//! entirely inside this crate — no service code needs to change.
//!
//! ## Level guide
//! ```text
//! RUST_LOG=debug   beta / local debug
//! RUST_LOG=info    MVP production (default)
//! RUST_LOG=warn    minimal — only retries, fallbacks, errors
//! ```

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes the global tracing subscriber for `service_name`.
///
/// Must be called once, before any `tracing::*!` macros. Panics if the
/// global subscriber is already set (called more than once per process).
pub fn init(service_name: &'static str) {
    // --- OTel tracer provider -----------------------------------------------
    // `TracerProvider::default()` creates a provider with no span exporter:
    // spans flow through the pipeline and are dropped. This gives us the full
    // instrumentation surface (context propagation, span creation, attributes)
    // without sending anything over the wire.
    //
    // To enable OTLP export, replace this block with:
    //
    //   use opentelemetry_otlp::WithExportConfig;
    //   use opentelemetry_sdk::{runtime, Resource};
    //   use opentelemetry::KeyValue;
    //
    //   let provider = opentelemetry_otlp::new_pipeline()
    //       .tracing()
    //       .with_exporter(
    //           opentelemetry_otlp::new_exporter()
    //               .tonic()
    //               .with_endpoint("http://otel-collector:4317"),
    //       )
    //       .with_trace_config(
    //           opentelemetry_sdk::trace::Config::default().with_resource(
    //               Resource::new(vec![KeyValue::new("service.name", service_name)]),
    //           ),
    //       )
    //       .install_batch(runtime::Tokio)
    //       .expect("failed to init OTLP tracer");
    //
    // and add to Cargo.toml:
    //   opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
    //   opentelemetry_sdk  = { version = "0.27", features = ["trace", "rt-tokio"] }
    // --------------------------------------------------------------------------
    let provider = TracerProvider::default();
    let tracer = provider.tracer(service_name);
    opentelemetry::global::set_tracer_provider(provider);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();
}

/// Flushes and shuts down the OTel tracer provider.
///
/// Call before process exit when using a real exporter to avoid dropping
/// in-flight spans. With the no-exporter provider (current default) this
/// is a no-op but harmless to call.
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}
