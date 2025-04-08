use super::types::{request, response};
use crate::{
    modules::{kitchen, meal::repository, storage},
    types::Context,
};
use std::{io::Read, sync::Arc};

pub async fn service(ctx: Arc<Context>, mut payload: request::Payload) -> response::Response {
    let kitchen =
        kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id.clone())
            .await
            .map_err(|_| response::Error::FailedToCreateMeal)?
            .ok_or(response::Error::KitchenNotCreated)?;

    let mut buf: Vec<u8> = vec![];

    payload
        .body
        .cover_image
        .contents
        .read_to_end(&mut buf)
        .map_err(|err| {
            tracing::error!("Failed to read the uploaded file {:?}", err);
            response::Error::FailedToCreateMeal
        })?;

    let cover_image = storage::upload_file(ctx.storage.clone(), buf)
        .await
        .map_err(|err| {
            tracing::error!("Failed to upload file: {:?}", err);
            response::Error::FailedToCreateMeal
        })?;

    repository::create(
        &ctx.db_conn.pool,
        repository::CreateMealPayload {
            name: payload.body.name,
            description: payload.body.description,
            price: payload.body.price.0,
            cover_image,
            kitchen_id: kitchen.id,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateMeal)
    .map(response::Success::MealCreated)
}
