mod create;
mod delete;
mod get;
mod like;
mod list;
mod unlike;
mod update;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", create::get_router())
        .nest("/", list::get_router())
        .nest("/", delete::get_router())
        .nest("/", get::get_router())
        .nest("/", update::get_router())
        .nest("/", like::get_router())
        .nest("/", unlike::get_router())
}
