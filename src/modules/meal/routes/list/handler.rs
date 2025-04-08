use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: Option<Auth>,
    Query(filters): Query<request::Filters>,
    pagination: Pagination,
) -> impl IntoResponse {
    service(
        ctx,
        request::Payload {
            auth,
            pagination,
            filters,
        },
    )
    .await
}
