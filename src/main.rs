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
    let ctx: Arc<Context> = Arc::new(Config::default().to_context().await);

    init_tracing();

    let app = App::new().await;

    let http = app.serve();
    let job_monitor = async { jobs::monitor(ctx.clone()).await.run().await.unwrap() };

    tokio::join!(http, job_monitor);
}
