use std::net::SocketAddr;

use axum::{routing::get, Router};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::trace::Tracer;
use opentelemetry::trace::{TraceContextExt, TraceError};
use opentelemetry::Context;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry_http::HeaderInjector;
use tracing_subscriber::EnvFilter;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint("http://127.0.0.1:14268/api/traces")
        .with_service_name("awesome-api")
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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await;

    shutdown_tracer_provider();

    Ok(())
}

async fn index() -> String {
    let tracer = global::tracer("request");

    let span = tracer.start("HTTP GET /");
    let cx = Context::current_with_span(span);

    let child_span = tracer.start_with_context("Call repeater-api", &cx);
    let child_cx = Context::current_with_span(child_span);

    let url = reqwest::Url::parse("http://localhost:4000/").unwrap();
    let mut req = reqwest::Request::new(reqwest::Method::GET, url);

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&child_cx, &mut HeaderInjector(req.headers_mut()))
    });

    let client = reqwest::Client::new();
    let res = client.execute(req).await.unwrap().text().await.unwrap();

    child_cx.span().end();

    cx.span().end();

    format!("Hello, World!: {}", res)
}
