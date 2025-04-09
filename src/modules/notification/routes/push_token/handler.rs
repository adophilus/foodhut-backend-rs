use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(body): Json<request::Body>,
) -> impl IntoResponse {
    service(ctx, request::Payload { body, auth }).await
}
