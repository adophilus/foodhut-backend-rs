use super::types::{request, response};
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::delete_city_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToDeleteCity)
        .map(|_| response::Success::CityDeleted)
}
