use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Path(meal_id): Path<String>,
) -> impl IntoResponse {
    service(ctx, auth, request::Payload { meal_id }).await
}
