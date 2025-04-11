use super::service::service;
use super::types::request;
use crate::types::Context;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: request::Auth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id, auth }).await
}
