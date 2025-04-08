mod block;
mod cities;
mod get;
mod items;
mod like;
mod types;
mod unblock;
mod unlike;
mod update;
mod update_cover_image;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", get::get_router())
        .nest("/", update::get_router())
        .nest("/", update_cover_image::get_router())
        .nest("/", like::get_router())
        .nest("/", unlike::get_router())
        .nest("/", types::get_router())
        .nest("/", block::get_router())
        .nest("/", unblock::get_router())
        .nest("/", cities::get_router())
        .nest("/", items::get_router())
}
