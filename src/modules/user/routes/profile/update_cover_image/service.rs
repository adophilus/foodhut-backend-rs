use super::super::super::super::repository;
use super::types::{request, response};
use crate::{modules::storage, types::Context};
use std::{io::Read, sync::Arc};

pub async fn service(ctx: Arc<Context>, mut payload: request::Payload) -> response::Response {
    let mut buf: Vec<u8> = vec![];

    payload
        .body
        .profile_picture
        .contents
        .read_to_end(&mut buf)
        .map_err(|err| {
            tracing::error!("Failed to read the uploaded file {:?}", err);
            response::Error::FailedToUpdateProfilePicture
        })?;

    let profile_picture = match payload.auth.user.profile_picture.0 {
        Some(profile_picture) => {
            storage::update_file(ctx.storage.clone(), profile_picture, buf).await
        }
        None => storage::upload_file(ctx.storage.clone(), buf).await,
    }
    .map_err(|_| response::Error::FailedToUpdateProfilePicture)?;

    repository::update_by_id(
        &ctx.db_conn.pool,
        payload.auth.user.id,
        repository::UpdateUserPayload {
            email: None,
            phone_number: None,
            first_name: None,
            last_name: None,
            has_kitchen: None,
            profile_picture: Some(profile_picture),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateProfilePicture)
    .map(|_| response::Success::ProfilePictureUpdated)
}
