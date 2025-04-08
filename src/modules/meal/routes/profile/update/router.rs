use super::handler;
use crate::types::Context;
use axum::routing::{patch, Router};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", patch(handler::handler))
}
