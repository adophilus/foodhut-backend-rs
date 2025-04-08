mod get;
mod update;
mod update_cover_image;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest(
        "/profile",
        Router::new()
            .nest("/", get::get_router())
            .nest("/", update::get_router())
            .nest("/", update_cover_image::get_router()),
    )
}
