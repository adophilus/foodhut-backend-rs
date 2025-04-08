use super::types::{request, response};
use crate::{
    modules::{kitchen::repository, storage},
    types::Context,
};
use std::{io::Read, sync::Arc};

pub async fn service(ctx: Arc<Context>, mut payload: request::Payload) -> response::Response {
    let kitchen = repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToFetchKitchen)?
        .ok_or(response::Error::KitchenNotFound)?;

    if !repository::is_owner(&payload.auth.user, &kitchen) {
        return Err(response::Error::NotKitchenOwner);
    }

    let mut buf: Vec<u8> = vec![];

    payload
        .body
        .cover_image
        .contents
        .read_to_end(&mut buf)
        .map_err(|err| {
            tracing::error!("Failed to read the uploaded file {:?}", err);
            response::Error::FailedToUpdateCoverImage
        })?;

    let cover_image = match kitchen.cover_image.0 {
        Some(cover_image) => storage::update_file(ctx.storage.clone(), cover_image, buf).await,
        None => storage::upload_file(ctx.storage.clone(), buf).await,
    }
    .map_err(|_| response::Error::FailedToUpdateCoverImage)?;

    repository::update_by_id(
        &ctx.db_conn.pool,
        kitchen.id,
        repository::UpdateKitchenPayload {
            name: None,
            address: None,
            phone_number: None,
            r#type: None,
            opening_time: None,
            closing_time: None,
            preparation_time: None,
            delivery_time: None,
            cover_image: Some(cover_image),
            rating: None,
            likes: None,
            is_available: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateCoverImage)
    .map(|_| response::Success::CoverImageUpdated)
}
