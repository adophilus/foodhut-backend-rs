use axum::Router;

use super::paystack;
use crate::types::Context;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest("/paystack", paystack::get_router())
}
