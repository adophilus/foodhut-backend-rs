mod create;
mod details;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest(
        "/bank-account",
        Router::new()
            .nest("/", create::get_router())
            .nest("/", details::get_router()),
    )
}
