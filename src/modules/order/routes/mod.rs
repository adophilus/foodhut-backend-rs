mod get;
mod list;
mod pay;
mod update_status;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", list::get_router())
        .nest("/", get::get_router())
        .nest("/", pay::get_router())
        .nest("/", update_status::get_router())
}
