use super::service::service;
use super::types::request;
use crate::{types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    pagination: Pagination,
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<request::Filters>,
) -> impl IntoResponse {
    service(
        ctx,
        request::Payload {
            pagination,
            filters,
        },
    )
    .await
}
