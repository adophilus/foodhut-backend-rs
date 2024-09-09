use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use super::{ad, auth, cart, dashboard, kitchen, meal, media, notification, order, user, webhooks};
use crate::types::Context;
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
        .nest("/ads", ad::get_router())
        .nest("/auth", auth::get_router())
        .nest("/media", media::get_router())
        .nest("/users", user::get_router())
        .nest("/kitchens", kitchen::get_router())
        .nest("/meals", meal::get_router())
        .nest("/carts", cart::get_router())
        .nest("/orders", order::get_router())
        .nest("/webhooks", webhooks::get_router())
        .nest("/notifications", notification::get_router())
        .nest("/dashboard", dashboard::get_router())
    // .nest("/wallets", wallet::get_router())
    // .layer(axum::middleware::from_fn(auth::middleware::auth))
}
