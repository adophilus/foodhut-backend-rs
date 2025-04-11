pub mod oauthredirect;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().nest("/", oauthredirect::get_router())
}
