use super::service::service;
use axum::response::IntoResponse;

pub async fn handler() -> impl IntoResponse {
    service()
}
