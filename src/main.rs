mod api;
mod repository;
mod types;
mod utils;

use crate::utils::{config, database};
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

async fn make_context(config: config::Config) -> types::Context {
    let db_conn = database::connect(config.database.url.as_str()).await;
    database::migrate(db_conn.clone()).await;

    types::Context {
        app: types::AppContext {
            host: config.app.host,
            port: config.app.port,
            url: config.app.url,
        },
        db_conn: db_conn.clone(),
        storage: types::StorageContext {
            api_key: config.storage.api_key,
            api_secret: config.storage.api_secret,
            upload_endpoint: config.storage.upload_endpoint,
            delete_endpoint: config.storage.delete_endpoint,
            upload_preset: config.storage.upload_preset,
        },
    }
}

#[tokio::main]
async fn main() {
    let config = config::get_config();
    let ctx = Arc::new(make_context(config.clone()).await);

    init_tracing();

    let app = Router::new()
        .nest("/api", api::get_router())
        .with_state(ctx.clone())
        .layer(Extension(ctx))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .layer(TraceLayer::new_for_http());

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))
            .await
            .unwrap();

    tracing::debug!("App is running on {}:{}", config.app.host, config.app.port);

    axum::serve(listener, app).await.unwrap();
}
