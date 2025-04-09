mod destroy;
mod get;
mod upload;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", upload::get_router())
        .nest("/", get::get_router())
        .nest("/", destroy::get_router())
}
