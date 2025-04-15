use super::{
    service::service,
    types::{request, response},
};
use crate::{modules::auth::middleware::AdminAuth, types::Context, utils::pagination::Pagination};
use axum::extract::{Query, State};
use std::sync::Arc;

pub async fn handler(
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<request::Filters>,
    pagination: Pagination,
) -> response::Response {
    service(
        ctx,
        request::Payload {
            filters,
            pagination,
        },
    )
    .await
}
