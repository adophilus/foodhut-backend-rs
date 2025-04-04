mod sign_in;

use axum::routing::Router;
use std::sync::Arc;
use crate::types::Context;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest("/sign-in", sign_in::get_router())
}
