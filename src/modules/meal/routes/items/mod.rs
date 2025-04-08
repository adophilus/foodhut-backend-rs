mod remove_meal;
mod set_meal;

use axum::routing::Router;
use crate::types::Context;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest(
        "/items",
        Router::new()
            .nest("/", set_meal::get_router())
            .nest("/", remove_meal::get_router()),
    )
}
