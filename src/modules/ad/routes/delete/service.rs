use super::types::{request, response};
use crate::{
    modules::{ad::repository, storage},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let ad = repository::find_by_id(&ctx.db_conn.pool, payload.id.clone())
        .await
        .map_err(|_| response::Error::FailedToDeleteAd)?
        .ok_or(response::Error::AdNotFound)?;

    storage::delete_file(ctx.storage.clone(), ad.banner_image.clone())
        .await
        .map_err(|_| response::Error::FailedToDeleteAd)?;

    repository::delete_by_id(&ctx.db_conn.pool, payload.id.clone())
        .await
        .map_err(|_| response::Error::FailedToDeleteAd)
        .map(|_| return response::Success::AdDeleted)
}
