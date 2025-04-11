use super::types::{request, response};
use crate::{modules::ad::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::find_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToFetchAd)?
        .ok_or(response::Error::AdNotFound)
        .map(response::Success::Ad)
}
