use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, kitchen::repository, user, wallet},
    types::Context,
};
use std::sync::Arc;
use validator::Validate;

pub async fn service(
    ctx: Arc<Context>,
    auth: Auth,
    payload: request::Payload,
) -> response::Response {
    payload.validate().map_err(|errors| {
        tracing::warn!("Failed to validate payload: {errors}");
        response::Error::FailedToValidate(errors)
    })?;

    let mut tx = ctx.db_conn.clone().pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {}", err);
        response::Error::FailedToCreateKitchen
    })?;

    let kitchen = repository::find_by_owner_id(&mut *tx, auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToCreateKitchen)?;

    if kitchen.is_some() {
        return Err(response::Error::AlreadyCreatedKitchen);
    }

    repository::create(
        &mut *tx,
        repository::CreateKitchenPayload {
            name: payload.name,
            address: payload.address,
            r#type: payload.type_,
            phone_number: payload.phone_number,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            city_id: payload.city_id,
            owner_id: auth.user.id.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateKitchen)?;

    user::repository::update_by_id(
        &mut *tx,
        auth.user.id.clone(),
        user::repository::UpdateUserPayload {
            has_kitchen: Some(true),
            email: None,
            last_name: None,
            first_name: None,
            phone_number: None,
            profile_picture: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateKitchen)?;

    wallet::repository::create(
        &mut *tx,
        wallet::repository::CreateWalletPayload {
            is_kitchen_wallet: true,
            owner_id: auth.user.id.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateKitchen)?;

    tx.commit()
        .await
        .map_err(|err| {
            tracing::error!("Failed to commit database transaction: {}", err);
            response::Error::FailedToCreateKitchen
        })
        .map(|_| response::Success::KitchenCreated)
}
