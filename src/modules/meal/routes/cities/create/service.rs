use super::types::{request, response};
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::create_kitchen_city(
        &ctx.db_conn.pool,
        repository::CreateCityPayload {
            name: payload.name,
            state: payload.state,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateCity)
    .map(|_| response::Success::CityCreated)
}
