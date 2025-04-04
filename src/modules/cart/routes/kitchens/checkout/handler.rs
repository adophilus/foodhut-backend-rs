use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    auth: Auth,
    Path(kitchen_id): Path<String>,
    State(ctx): State<Arc<Context>>,
    Json(body): Json<request::Body>,
) -> impl IntoResponse {
    service(ctx, auth, request::Payload { body, kitchen_id }).await
}
