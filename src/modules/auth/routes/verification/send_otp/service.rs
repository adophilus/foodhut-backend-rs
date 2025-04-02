use super::types::{request, response};
use crate::{
    modules::{auth::service, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let user =
        user::repository::find_by_phone_number(&ctx.db_conn.pool, payload.phone_number.clone())
            .await
            .map_err(|_| response::Error::FailedToFetchUser)?
            .ok_or(response::Error::UserNotFound)?;

    if user.is_verified {
        return Err(response::Error::UserAlreadyVerified);
    }

    service::otp::send(ctx.clone(), user, "auth.verification".to_string())
        .await
        .map(|_| response::Success::CheckPhoneForVerificationOtp)
        .map_err(|err| match err {
            service::otp::SendError::NotExpired => response::Error::OtpNotExpired,
            _ => response::Error::FailedToSendOtp,
        })
}
