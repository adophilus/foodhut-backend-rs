use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Path(payload): Path<request::Payload>,
) -> impl IntoResponse {
    service(ctx, payload).await
}
