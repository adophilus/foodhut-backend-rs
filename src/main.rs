mod app;
mod jobs;
mod modules;
mod types;
mod utils;

use crate::{
    app::App,
    types::{Config, Context, ToContext},
};
use std::sync::Arc;
use tracing_subscriber::prelude::*;

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    let ctx: Arc<Context> = Arc::new(Config::default().to_context().await);

    let app = App::new(ctx.clone()).await;

    let http = app.serve();
    let job_monitor = async { jobs::monitor(ctx).await.run().await.unwrap() };

    tokio::join!(http, job_monitor);
}
