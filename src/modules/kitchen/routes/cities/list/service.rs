use super::types::response;
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>) -> response::Response {
    repository::find_many_cities(&ctx.db_conn.pool)
        .await
        .map_err(|_| response::Error::FailedToFetchCities)
        .map(response::Success::Cities)
}
