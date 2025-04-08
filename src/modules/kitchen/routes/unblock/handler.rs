use super::{service::service, types::request};
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id }).await
}
