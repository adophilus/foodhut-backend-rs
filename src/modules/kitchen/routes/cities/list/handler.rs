use super::service::service;
use crate::{modules::auth::middleware::AdminAuth, types::Context};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(_: AdminAuth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    service(ctx).await
}
