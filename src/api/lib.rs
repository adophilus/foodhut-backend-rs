use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::{
    // api::{auth, cart, kitchen, meal, order, user, wallet},
    api::{auth, cart, kitchen, meal, user},
    types::{ApiResponse, Context},
};
use std::sync::Arc;

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "message":"Welcome to Foodhut API"})),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(health_check))
        .nest("/auth", auth::get_router())
        .nest("/users", user::get_router())
        .nest("/kitchens", kitchen::get_router())
        .nest("/meals", meal::get_router())
        .nest("/carts", cart::get_router())
    // .nest("/orders", order::get_router())
    // .nest("/wallets", wallet::get_router())
    // .layer(axum::middleware::from_fn(auth::middleware::auth))
}
