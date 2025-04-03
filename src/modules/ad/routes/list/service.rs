use super::types::{request, response};
use crate::{
    modules::{ad::repository, storage},
    types::Context,
};
use std::io::Read;
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::find_many(&ctx.db_conn.pool, pagination.clone(), filters)
        .await
        .map(response::Success::PaginatedAds)
        .map_err(|_| response::Error::FailedToFetchAds)
}
