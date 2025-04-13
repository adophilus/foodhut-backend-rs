mod generate_token;
mod register_user;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/generate-token", generate_token::get_router())
        .nest("/register-user", register_user::get_router())
}
