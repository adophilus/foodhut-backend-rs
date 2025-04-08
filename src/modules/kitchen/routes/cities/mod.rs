mod create;
mod delete;
mod get;
mod update;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest(
        "/cities",
        Router::new()
            .nest("/", get::get_router())
            .nest("/", create::get_router())
            .nest("/", update::get_router())
            .nest("/", delete::get_router()),
    )
}
