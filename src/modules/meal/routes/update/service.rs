use super::types::{request, response};
use crate::{
    modules::{
        kitchen,
        meal::repository,
        storage::{self, UploadedMedia},
    },
    types::{Context, StorageContext},
};
use axum_typed_multipart::FieldData;
use std::{io::Read, sync::Arc};
use tempfile::NamedTempFile;

async fn upload_cover_image(
    storage: StorageContext,
    mut new_image: FieldData<NamedTempFile>,
    old_image: UploadedMedia,
) -> Result<UploadedMedia, response::Error> {
    let mut buf: Vec<u8> = vec![];

    new_image.contents.read_to_end(&mut buf).map_err(|err| {
        tracing::error!("Failed to read the uploaded file {err:?}");
        response::Error::FailedToUpdateMeal
    })?;

    storage::update_file(storage, old_image, buf)
        .await
        .map_err(|_| response::Error::FailedToUpdateMeal)
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let kitchen =
        kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.clone().id)
            .await
            .map_err(|_| response::Error::FailedToUpdateMeal)?
            .ok_or(response::Error::KitchenNotCreated)?;

    let meal = repository::find_by_id(&ctx.db_conn.pool, payload.id.clone())
        .await
        .map_err(|_| response::Error::FailedToUpdateMeal)?
        .ok_or(response::Error::MealNotFound)?;

    if !repository::is_owner(&payload.auth.user, &kitchen, &meal) {
        return Err(response::Error::NotMealOwner);
    }

    let cover_image = match payload
        .body
        .cover_image
        .map(|image| upload_cover_image(ctx.storage.clone(), image, meal.cover_image))
    {
        Some(fut) => Some(fut.await),
        _ => None,
    }
    .transpose()?;

    repository::update_by_id(
        &ctx.db_conn.pool,
        payload.id,
        repository::UpdateMealPayload {
            name: payload.body.name,
            description: payload.body.description,
            price: payload.body.price.map(|price| price.0),
            rating: None,
            is_available: payload.body.is_available,
            cover_image,
            kitchen_id: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateMeal)
    .map(|_| response::Success::MealUpdated)
}
