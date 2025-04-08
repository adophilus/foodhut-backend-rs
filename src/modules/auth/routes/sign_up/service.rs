use super::types::{request, response};
use crate::{
    modules::{auth::service, notification, user, wallet},
    types::Context,
};
use std::sync::Arc;
use validator::Validate;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    payload.validate().map_err(|errors| {
        tracing::warn!("Failed to validate payload: {errors}");
        response::Error::FailedToValidate(errors)
    })?;

    let mut tx = ctx.db_conn.clone().pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {}", err);
        return response::Error::UnexpectedError;
    })?;

    match user::repository::find_by_email_or_phone_number(
        &mut *tx,
        user::repository::FindByEmailOrPhoneNumber {
            email: payload.email.clone().to_lowercase(),
            phone_number: payload.phone_number.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToFetchUser)?
    {
        Some(user) => {
            if user.email == payload.email {
                return Err(response::Error::EmailAlreadyInUse);
            }
            return Err(response::Error::PhoneNumberAlreadyInUse);
        }
        _ => (),
    };

    let user = user::repository::create(
        &mut *tx,
        user::repository::CreateUserPayload {
            email: payload.email.clone(),
            phone_number: payload.phone_number.clone(),
            first_name: payload.first_name.clone(),
            last_name: payload.last_name.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::SignupFailed)?;

    // TODO: Notification failing to send is insignificant for now
    let _ = tokio::spawn(notification::service::send(
        ctx.clone(),
        notification::service::Notification::registered(user.clone()),
        notification::service::Backend::Email,
    ));

    wallet::repository::create(
        &mut *tx,
        wallet::repository::CreateWalletPayload {
            owner_id: user.id.clone(),
            is_kitchen_wallet: false,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateWallet)?;

    tx.commit().await.map_err(|err| {
        tracing::error!("Failed to commit database transaction: {}", err);
        return response::Error::UnexpectedError;
    })?;

    // TODO: if an error occurs at this point the user can always request for another OTP
    tokio::spawn(service::otp::send(
        ctx.clone(),
        user,
        "auth.verification".to_string(),
    ));

    return Ok(response::Success::CheckPhoneForVerificationOtp);
}
