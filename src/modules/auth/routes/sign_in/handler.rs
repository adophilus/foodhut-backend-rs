use super::service::service;
use super::types::request;
use crate::types::Context;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<request::Payload>,
) -> impl IntoResponse {
    service(ctx, payload).await
}
