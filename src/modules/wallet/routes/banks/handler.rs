use super::{service::service, types::request};
use crate::types::Context;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    pagination: request::Pagination,
) -> impl IntoResponse {
    service(ctx, request::Payload { pagination }).await
}
