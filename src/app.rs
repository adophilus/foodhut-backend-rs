use crate::{
    modules,
    types::{Config, Context, ToContext},
};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Extension, Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors, trace};
use tracing_subscriber::prelude::*;

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

pub struct App {
    ctx: Arc<Context>,
    router: Router,
}

impl App {
    pub async fn new() -> Self {
        let ctx: Arc<Context> = Arc::new(Config::default().to_context().await);

        let router = Router::new()
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

        Self { ctx, router }
    }

    pub async fn serve(self) {
        let listener = TcpListener::bind(format!("{}:{}", self.ctx.app.host, self.ctx.app.port))
            .await
            .unwrap();

        axum::serve(listener, self.router).await.unwrap();

        tracing::debug!("App is running on {}:{}", self.ctx.app.host, self.ctx.app.port);
    }
}
