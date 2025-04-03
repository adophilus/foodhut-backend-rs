use axum_typed_multipart::FieldData;
use tempfile::NamedTempFile;

use super::types::{request, response};
use crate::{
    modules::{
        ad::repository,
        storage::{self, UploadedMedia},
    },
    types::{Context, StorageContext},
};
use std::{io::Read, sync::Arc};

async fn upload_banner_image(
    storage: StorageContext,
    mut new_image: FieldData<NamedTempFile>,
    old_image: UploadedMedia,
) -> Result<UploadedMedia, response::Error> {
    let mut buf: Vec<u8> = vec![];

    new_image.contents.read_to_end(&mut buf).map_err(|err| {
        tracing::error!("Failed to read the uploaded file {err:?}");
        response::Error::FailedToUploadImage
    })?;

    storage::update_file(storage, old_image, buf)
        .await
        .map_err(|_| response::Error::FailedToUploadImage)
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let ad = repository::find_by_id(&ctx.db_conn.pool, payload.id.clone())
        .await
        .map_err(|_| response::Error::FailedToFetchAd)?
        .ok_or(response::Error::AdNotFound)?;

    let banner_image = match payload.body.banner_image.map(|banner_image| {
        upload_banner_image(ctx.storage.clone(), banner_image, ad.banner_image.clone())
    }) {
        Some(fut) => Some(fut.await),
        _ => None,
    }
    .transpose()?;

    repository::update_by_id(
        &ctx.db_conn.pool,
        payload.id,
        repository::UpdateAdPayload {
            banner_image,
            link: payload.body.link,
            duration: payload.body.duration,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateAd)
    .map(|_| response::Success::AdUpdated)
}
