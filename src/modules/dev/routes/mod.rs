mod auth;
mod test;
mod zoho;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/auth", auth::get_router())
        .nest("/test", test::get_router())
        .nest("/zoho", zoho::get_router())
}
