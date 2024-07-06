mod api;
mod repository;
mod types;
mod utils;

use crate::utils::{config, database};
use axum::{Extension, Router};
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
    let config = config::get_config();
    let db_conn = database::connect(config.database.url.as_str()).await;
    let ctx = Arc::new(types::Context { db_conn: db_conn.clone() });

    init_tracing();
    database::migrate(db_conn.clone()).await;

    let app = Router::new()
        .nest("/api", api::get_router())
        .with_state(ctx.clone())
        .layer(Extension(ctx))
        .layer(TraceLayer::new_for_http());

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
            .await
            .unwrap();

    axum::serve(listener, app).await.unwrap();
}
