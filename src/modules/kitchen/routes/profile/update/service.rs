use super::types::{request, response};
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let kitchen = repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToFetchKitchen)?
        .ok_or(response::Error::KitchenNotFound)?;

    if !repository::is_owner(&payload.auth.user, &kitchen) {
        return Err(response::Error::NotKitchenOwner);
    }

    repository::update_by_id(
        &ctx.db_conn.pool,
        kitchen.id,
        repository::UpdateKitchenPayload {
            name: payload.body.name,
            address: payload.body.address,
            phone_number: payload.body.phone_number,
            r#type: payload.body.r#type,
            opening_time: payload.body.opening_time,
            closing_time: payload.body.closing_time,
            preparation_time: payload.body.preparation_time,
            delivery_time: payload.body.delivery_time,
            cover_image: None,
            rating: None,
            likes: None,
            is_available: payload.body.is_available,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateKitchen)
    .map(|_| response::Success::KitchenUpdated)
}
