use super::{service::service, types::request};
use crate::types::Context;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: request::Auth,
    State(ctx): State<Arc<Context>>,
    Query(params): Query<request::Params>,
) -> impl IntoResponse {
    service(ctx, request::Payload { params }).await
}
