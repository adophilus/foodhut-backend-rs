use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{extract::State, response::IntoResponse};
use axum_typed_multipart::TypedMultipart;
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    TypedMultipart(payload): TypedMultipart<request::Payload>,
) -> impl IntoResponse {
    service(ctx, payload).await
}
