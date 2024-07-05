use crate::types::Context;
use crate::utils::database::DatabaseConnection;
use crate::{repository, types::ApiResponse};
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct SignUpPayload {
    email: String,
    phone_number: String,
    first_name: String,
    last_name: String,
    birthday: NaiveDate,
}

async fn sign_up(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignUpPayload>,
) -> axum::Json<ApiResponse<&'static str, &'static str>> {
    match (
        repository::user::find_by_email(ctx.db_conn.clone(), payload.email.clone()).await,
        repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
            .await,
    ) {
        (None, None) => {
            let res = repository::user::create(
                ctx.db_conn.clone(),
                repository::user::CreateUserPayload {
                    email: payload.email.clone(),
                    phone_number: payload.phone_number.clone(),
                    first_name: payload.first_name.clone(),
                    last_name: payload.last_name.clone(),
                    birthday: payload.birthday.clone(),
                },
            )
            .await;

            match res {
                Ok(_) => axum::Json(ApiResponse::ok("Sign up successful")),
                Err(_) => axum::Json(ApiResponse::err("Sign up failed!")),
            }
        }
        (Some(_), _) => axum::Json(ApiResponse::err("Email taken")),
        (_, Some(_)) => axum::Json(ApiResponse::err("Phone number taken")),
    }
}

#[derive(Deserialize)]
struct SendVerificationOtpPayload {
    phone_number: String,
}

async fn verification_send_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SendVerificationOtpPayload>,
) -> axum::Json<ApiResponse<&'static str, &'static str>> {
    match repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
        .await
    {
        Some(user) => {
            if user.is_verified {
                return axum::Json(ApiResponse::err("User already verified"));
            }
        }
        None => {
            return axum::Json(ApiResponse::err("User not found"));
        }
    }

    match repository::otp::create(
        ctx.db_conn.clone(),
        "auth.sign-up.verification".to_string(),
        payload.phone_number,
    )
    .await
    {
        Ok(otp) => {
            // TODO: actually send the OTP using twilio or something
            axum::Json(ApiResponse::ok("OTP sent!"))
        }
        Err(repository::otp::Error::OtpNotExpired) => {
            axum::Json(ApiResponse::err("OTP not expired"))
        }
        Err(_) => axum::Json(ApiResponse::err("Failed to send OTP")),
    }
}

#[derive(Deserialize)]
struct VerifyOtpPayload {
    phone_number: String,
    otp: String,
}

async fn verification_verify_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<VerifyOtpPayload>,
) -> axum::Json<ApiResponse<&'static str, &'static str>> {
    match repository::otp::verify(
        ctx.db_conn.clone(),
        "auth.sign-up.verification".to_string(),
        payload.phone_number.clone(),
        payload.otp.clone(),
    )
    .await
    {
        Ok(_) => {
            match repository::user::verify_by_phone_number(
                ctx.db_conn.clone(),
                payload.phone_number,
            )
            .await
            {
                Ok(_) => axum::Json(ApiResponse::ok("OTP verified")),
                _ => axum::Json(ApiResponse::err("Failed to verify OTP")),
            }
        }
        _ => axum::Json(ApiResponse::err("Invalid OTP")),
    }
}

#[derive(Deserialize)]
struct SignInSendOtpPayload {
    phone_number: String,
}

async fn sign_in_send_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignInSendOtpPayload>,
) -> axum::Json<ApiResponse<&'static str,  &'static str>> {
    match repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
        .await
    {
        Some(user) => {
            if !user.is_verified {
                return axum::Json(ApiResponse::err("User not verified"));
            }
        }
        None => {
            return axum::Json(ApiResponse::err("User not found"));
        }
    }

    match repository::otp::create(
        ctx.db_conn.clone(),
        "auth.sign-in.verification".to_string(),
        payload.phone_number,
    )
    .await
    {
        Ok(otp) => {
            // TODO: actually send the OTP using twilio or something
            axum::Json(ApiResponse::ok("OTP sent!"))
        }
        Err(repository::otp::Error::OtpNotExpired) => {
            axum::Json(ApiResponse::err("OTP not expired"))
        }
        Err(_) => axum::Json(ApiResponse::err("Failed to send OTP")),
    }
}

#[derive(Deserialize)]
struct SignInVerifyOtpPayload {
    phone_number: String,
    otp: String,
}

#[derive(Serialize)]
struct SessionTokenPayload {
    token: String,
}

async fn sign_in_verify_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignInVerifyOtpPayload>,
) -> axum::Json<ApiResponse<SessionTokenPayload, String>> {
    match repository::otp::verify(
        ctx.db_conn.clone(),
        "auth.sign-in.verification".to_string(),
        payload.phone_number.clone(),
        payload.otp.clone(),
    )
    .await
    {
        Ok(_) => {
            match repository::user::verify_by_phone_number(
                ctx.db_conn.clone(),
                payload.phone_number.clone(),
            )
            .await
            {
                Ok(_) => {
                    match repository::user::find_by_phone_number(
                        ctx.db_conn.clone(),
                        payload.phone_number.clone(),
                    )
                    .await
                    {
                        Some(user) => {
                            match repository::session::create(ctx.db_conn.clone(), user.id).await {
                                Ok(session) => axum::Json(ApiResponse::ok(SessionTokenPayload {
                                    token: session.id,
                                })),
                                Err(_) => {
                                    axum::Json(ApiResponse::err("Failed to create session".to_string()))
                                }
                            }
                        }
                        None => axum::Json(ApiResponse::err("User does not exist".to_string())),
                    }
                }
                _ => axum::Json(ApiResponse::err("Failed to verify OTP".to_string())),
            }
        }
        _ => axum::Json(ApiResponse::err("Invalid OTP".to_string())),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/sign-up/strategy/credentials", post(sign_up))
        .route("/verification/send-otp", post(verification_send_otp))
        .route("/verification/verify-otp", post(verification_verify_otp))
        .route("/sign-in/strategy/phone/send-otp", post(sign_in_send_otp))
        .route(
            "/sign-in/strategy/phone/verify-otp",
            post(sign_in_verify_otp),
        )
}
