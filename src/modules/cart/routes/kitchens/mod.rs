mod checkout;
mod remove_meals;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest(
        "/kitchens",
        Router::new()
            .nest("/", checkout::get_router())
            .nest("/", remove_meals::get_router()),
    )
}
