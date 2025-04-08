use super::types::{request, response};
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::update_city_by_id(
        &ctx.db_conn.pool,
        payload.id,
        repository::UpdateCityByIdPayload {
            name: payload.body.name,
            state: payload.body.state,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateCity)
    .map(|_| response::Success::CityUpdated)
}
