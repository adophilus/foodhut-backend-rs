mod block;
mod cities;
mod create;
mod get;
mod like;
mod list;
mod profile;
mod types;
mod unblock;
mod unlike;
mod update;
mod update_cover_image;
mod verify;
mod unverify;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", create::get_router())
        .nest("/", list::get_router())
        .nest("/", get::get_router())
        .nest("/", update::get_router())
        .nest("/", update_cover_image::get_router())
        .nest("/", like::get_router())
        .nest("/", unlike::get_router())
        .nest("/", types::get_router())
        .nest("/", profile::get_router())
        .nest("/", block::get_router())
        .nest("/", unblock::get_router())
        .nest("/", cities::get_router())
        .nest("/", verify::get_router())
        .nest("/", unverify::get_router())
}
