mod sign_in;
mod sign_up;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/sign-in", sign_in::get_router())
        .nest("/sign-up", sign_up::get_router())
}
