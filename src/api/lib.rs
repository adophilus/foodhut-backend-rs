use axum::{routing::get, Router};

use crate::{
    api::{auth, kitchen, user},
    types::{ApiResponse, Context},
};
use std::sync::Arc;

async fn health_check() -> ApiResponse<&'static str, &'static str> {
    ApiResponse::ok("We up!")
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(health_check))
        .nest("/auth", auth::get_router())
        .nest("/users", user::get_router())
        .nest("/kitchens", kitchen::get_router())
    // .layer(axum::middleware::from_fn(auth::middleware::auth))
}
