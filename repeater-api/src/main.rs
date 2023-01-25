use std::net::SocketAddr;

use axum::{http::HeaderMap, routing::get, Router};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::Tracer;
use opentelemetry::trace::{Span, Status, TraceError};
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry_http::HeaderExtractor;
use tracing_subscriber::EnvFilter;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint("http://127.0.0.1:14268/api/traces")
        .with_service_name("repeater-api")
        .with_reqwest()
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    init_tracer().expect("Failed to initialise tracer.");

    let app = Router::new().route("/", get(index));
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await;

    shutdown_tracer_provider();

    Ok(())
}

async fn index(headers: HeaderMap) -> &'static str {
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&headers))
    });

    let mut span = global::tracer("request").start_with_context("HTTP GET /", &parent_cx);
    span.set_status(Status::Ok);
    span.end();

    "Repeated"
}
