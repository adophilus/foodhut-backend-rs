use super::types::{request, response};
use crate::{
    modules::{ad::repository, storage},
    types::Context,
};
use std::io::Read;
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, mut payload: request::Payload) -> response::Response {
    let mut buf: Vec<u8> = vec![];

    payload
        .banner_image
        .contents
        .read_to_end(&mut buf)
        .map_err(|err| {
            tracing::error!("Failed to read the uploaded file {err:?}");
            response::Error::ImageUploadFailed
        })?;

    let banner_image = storage::upload_file(ctx.storage.clone(), buf)
        .await
        .map_err(|_| response::Error::ImageUploadFailed)?;

    repository::create(
        &ctx.db_conn.pool,
        repository::CreateAdPayload {
            banner_image,
            link: payload.link,
            duration: payload.duration,
        },
    )
    .await
    .map_err(|_| response::Error::AdCreationFailed)
    .map(response::Success::AdCreated)
}
