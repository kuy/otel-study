use std::net::SocketAddr;

use axum::{routing::get, Router};
use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::TraceError;
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Key,
};

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint("http://127.0.0.1:14268/api/traces")
        .with_service_name("awesome-api")
        .with_reqwest()
        .install_batch(opentelemetry::runtime::TokioCurrentThread)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracer().expect("Failed to initialise tracer.");

    let app = Router::new().route("/", get(index));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await;
    
    shutdown_tracer_provider();
    
    Ok(())
}

async fn index() -> &'static str {
    let tracer = global::tracer("request");
    tracer.in_span("index", |ctx| {
        ctx.span().set_attribute(Key::new("parameter").i64(10));
        "Hello, World!"
    })
}
