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

    if !user.is_verified {
        return Err(response::Error::UserNotVerified);
    }

    if user.deleted_at.is_some() {
        return Err(response::Error::UserNotFound);
    }

    let exempt_user = user::repository::find_exempt_by_id(&ctx.db_conn.pool, user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToSendOtp)?;

    if exempt_user.is_some() {
        service::otp::create(
            ctx.clone(),
            user,
            "auth.verification".to_string(),
            "1234".to_string(),
        )
        .await
        .map_err(|err| match err {
            service::otp::SendError::NotExpired => response::Error::OtpNotExpired,
            _ => response::Error::FailedToSendOtp,
        })?;

        return Ok(response::Success::CheckPhoneForVerificationOtp);
    }

    service::otp::send(ctx.clone(), user, "auth.verification".to_string())
        .await
        .map_err(|err| match err {
            service::otp::SendError::NotExpired => response::Error::OtpNotExpired,
            _ => response::Error::FailedToSendOtp,
        })?;

    return Ok(response::Success::CheckPhoneForVerificationOtp);
}
