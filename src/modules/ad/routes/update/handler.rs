use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_typed_multipart::TypedMultipart;
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Path(id): Path<String>,
    TypedMultipart(body): TypedMultipart<request::Body>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id, body }).await
}
