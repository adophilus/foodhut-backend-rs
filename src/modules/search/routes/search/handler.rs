use super::service::service;
use super::types::request;
use crate::types::Context;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<request::Filters>,
    pagination: request::Pagination,
) -> impl IntoResponse {
    service(
        ctx,
        request::Payload {
            filters,
            pagination,
        },
    )
    .await
}
