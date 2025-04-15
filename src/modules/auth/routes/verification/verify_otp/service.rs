use super::types::{request, response};
use crate::{
    modules::{auth::service, user, zoho},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let mut tx = ctx
        .db_conn
        .pool
        .begin()
        .await
        .map_err(|_| response::Error::UnexpectedError)?;

    let user = user::repository::find_by_phone_number(&mut *tx, payload.phone_number.clone())
        .await
        .map_err(|_| response::Error::FailedToFetchUser)?
        .ok_or(response::Error::UserNotFound)?;

    service::otp::verify(
        ctx.clone(),
        &mut tx,
        user.clone(),
        "auth.verification".to_string(),
        payload.otp.clone(),
    )
    .await
    .map_err(|_| response::Error::InvalidOrExpiredOtp)?;

    user::repository::verify_by_phone_number(&mut *tx, payload.phone_number)
        .await
        .map_err(|_| response::Error::OtpVerificationFailed)?;

    if user.is_verified == false {
        zoho::service::register_user(ctx.clone(), user.clone())
            .await
            .map_err(|_| response::Error::UnexpectedError)?;
    }

    let session = service::auth::create_session(&mut *tx, user.id)
        .await
        .map_err(|_| response::Error::FailedToCreateSession)?;

    tx.commit()
        .await
        .map(|_| response::Success::Tokens((session.access_token, session.refresh_token)))
        .map_err(|err| {
            tracing::error!("Failed to commit transaction: {}", err);
            response::Error::UnexpectedError
        })
}
