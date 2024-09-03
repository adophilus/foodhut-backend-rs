mod api;
mod jobs;
mod repository;
mod types;
mod utils;

use crate::types::{Config, Context, ToContext};
use axum::{extract::DefaultBodyLimit, Extension, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() {
    let ctx: Arc<Context> = Arc::new(Config::default().to_context().await);

    init_tracing();

    let app = Router::new()
        .nest("/api", api::get_router())
        .with_state(ctx.clone())
        .layer(Extension(ctx.clone()))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ctx.app.host, ctx.app.port))
        .await
        .unwrap();

    tracing::debug!("App is running on {}:{}", ctx.app.host, ctx.app.port);

    let http = async { axum::serve(listener, app).await.unwrap() };
    let job_monitor = async { jobs::monitor(ctx.clone()).await.run().await.unwrap() };

    let _res = tokio::join!(http, job_monitor);

    // Ok(())
}
