use super::types::response;
use crate::{modules::dashboard::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>) -> response::Response {
    repository::get_total_resources(&ctx.db_conn.pool)
        .await
        .map_err(|_| response::Error::FailedToFetchInfo)
        .map(response::Success::Info)
}
