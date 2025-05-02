use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, kitchen::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(
    ctx: Arc<Context>,
    auth: Option<Auth>,
    payload: request::Payload,
) -> response::Response {
    let mut filters = repository::FindManyFilters {
        search: payload.filters.search,
        r#type: payload.filters.r#type,
        queryer_role: repository::QueryerRole::User,
    };

    if auth.is_some() && user::repository::is_admin(&auth.unwrap().user) {
        filters.queryer_role = repository::QueryerRole::Admin;
    }

    repository::find_many(&ctx.db_conn.pool, payload.pagination.clone(), filters)
        .await
        .map_err(|_| response::Error::FailedToFetchKitchens)
        .map(response::Success::Kitchens)
}
