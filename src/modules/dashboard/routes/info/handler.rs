use super::service::service;
use crate::types::Context;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    service(ctx).await
}
