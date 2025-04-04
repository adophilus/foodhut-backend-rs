use super::service::service;
use super::types::request;
use crate::{modules::auth::middleware::AdminAuth, types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<request::Filters>,
    pagination: Pagination,
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
