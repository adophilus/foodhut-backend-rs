mod get;
mod items;
mod kitchens;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", get::get_router())
        .nest("/", items::get_router())
        .nest("/", kitchens::get_router())
}
