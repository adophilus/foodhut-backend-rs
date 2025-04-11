use super::handler;
use crate::types::Context;
use axum::routing::{delete, Router};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", delete(handler::handler))
}
