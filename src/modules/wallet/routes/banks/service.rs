use super::{
    super::super::repository,
    types::{request, response},
};
use crate::types::Context;
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::find_many_banks(&ctx.db_conn.pool, payload.pagination)
        .await
        .map_err(|_| response::Error::FailedToFetchBanks)
        .map(response::Success::Banks)
}
