use super::types::{request, response};
use crate::{
    modules::{ad::repository, storage},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let mut tx = ctx.db_conn.pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {:?}", err);
        response::Error::FailedToDeleteAccount
    })?;

    let kitchen = kitchen::repository::find_by_owner_id(&mut *tx, auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToDeleteAccount)?
        .ok_or(response::Error::AccountNotFound)?;

    kitchen::repository::update_by_id(
        &mut *tx,
        kitchen.id,
        kitchen::repository::UpdateKitchenPayload {
            name: None,
            address: None,
            phone_number: None,
            r#type: None,
            opening_time: None,
            closing_time: None,
            preparation_time: None,
            delivery_time: None,
            cover_image: None,
            rating: None,
            likes: None,
            is_available: Some(false),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToDeleteAccount)?;

    repository::delete_by_id(&mut *tx, auth.user.id).await
        .map_err(|_| response::Error:FailedToDeleteAccount)
        .map(|_| response::Success::AccountDeleted)?;

    tx.commit()
        .await
        .map(|_| response::Success::Successful)
        .map_err(|err| {
            tracing::error!("Failed to commit database transaction: {:?}", err);
            response::Error::ServerError
        })
}
