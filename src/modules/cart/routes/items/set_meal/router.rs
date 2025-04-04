use super::handler;
use crate::types::Context;
use axum::routing::{put, Router};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/:meal_id", put(handler::handler))
}
