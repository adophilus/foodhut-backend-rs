use super::{service::service, types::request};
use crate::types::Context;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: request::Auth,
    Json(body): Json<request::Body>,
) -> impl IntoResponse {
    service(ctx, request::Payload { body, auth }).await
}
