mod analytics;
mod info;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", analytics::get_router())
        .nest("/", info::get_router())
}
