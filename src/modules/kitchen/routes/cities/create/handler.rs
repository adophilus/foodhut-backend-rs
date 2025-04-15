use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<request::Payload>,
) -> impl IntoResponse {
    service(ctx, payload).await
}
