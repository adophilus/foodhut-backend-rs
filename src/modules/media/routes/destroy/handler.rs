use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: Auth,
    State(ctx): State<Arc<Context>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id }).await
}
