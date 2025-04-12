use super::{service::service, types::request};
use crate::types::Context;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: request::Auth,
    Query(filters): Query<request::Filters>,
) -> impl IntoResponse {
    service(ctx, request::Payload { filters, auth }).await
}
