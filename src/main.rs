mod jobs;
mod modules;
mod types;
mod utils;

use crate::types::{Config, Context, ToContext};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Extension, Router,
};
use std::sync::Arc;
use tower_http::{cors, trace};
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
        .nest("/api", modules::get_router())
        .with_state(ctx.clone())
        .layer(Extension(ctx.clone()))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .layer(trace::TraceLayer::new_for_http())
        .layer(
            cors::CorsLayer::new()
                .allow_methods([
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([header::CONTENT_TYPE])
                .allow_origin(cors::Any),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ctx.app.host, ctx.app.port))
        .await
        .unwrap();

    tracing::debug!("App is running on {}:{}", ctx.app.host, ctx.app.port);

    let http = async { axum::serve(listener, app).await.unwrap() };
    let job_monitor = async { jobs::monitor(ctx.clone()).await.run().await.unwrap() };

    tokio::join!(http, job_monitor);
}
