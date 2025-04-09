use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Query(filters): Query<request::Filters>,
    pagination: Pagination,
) -> impl IntoResponse {
    service(
        ctx,
        request::Payload {
            pagination,
            filters,
            auth,
        },
    )
    .await
}
