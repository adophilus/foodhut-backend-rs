use crate::repository;
use crate::types::Context;
use crate::utils;
use crate::utils::notification;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct SignUpPayload {
    #[validate(email(code = "INVALID_USER_EMAIL", message = "Invalid email address"))]
    email: String,
    phone_number: String,
    first_name: String,
    last_name: String,
    birthday: NaiveDate,
}

async fn sign_up(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignUpPayload>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return utils::validation::into_response(errors);
    }

    match (
        repository::user::find_by_email(ctx.db_conn.clone(), payload.email.clone()).await,
        repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
            .await,
    ) {
        (None, None) => (),
        (Some(_), _) => return (StatusCode::CONFLICT, Json(json!({"error": "Email taken"}))),
        (_, Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({ "error": "Phone number taken"})),
            )
        }
    };

    match repository::user::create(
        ctx.db_conn.clone(),
        repository::user::CreateUserPayload {
            email: payload.email.clone(),
            phone_number: payload.phone_number.clone(),
            first_name: payload.first_name.clone(),
            last_name: payload.last_name.clone(),
            birthday: payload.birthday.clone(),
        },
    )
    .await
    {
        Ok(user) => {
            notification::send(
                ctx.clone(),
                notification::Notification::registered(user),
                notification::Backend::Email,
            )
            .await;

            (
                StatusCode::CREATED,
                Json(json!({
                    "message": "Sign up successful"
                })),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Sign up failed!"
            })),
        ),
    }
}

#[derive(Deserialize)]
struct SendVerificationOtpPayload {
    phone_number: String,
}

async fn verification_send_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SendVerificationOtpPayload>,
) -> impl IntoResponse {
    match repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
        .await
    {
        Some(user) => {
            if user.is_verified {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error" : "User already verified"})),
                );
            }
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error" : "User not found"})),
            );
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
            (StatusCode::OK, Json(json!({ "message" :"OTP sent!"})))
        }
        Err(repository::otp::Error::OtpNotExpired) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error" : "OTP not expired"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error" : "Failed to send OTP"})),
        ),
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
) -> impl IntoResponse {
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
                Ok(_) => (StatusCode::OK, Json(json!({ "message" : "OTP verified"}))),
                _ => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error" : "Failed to verify OTP"})),
                ),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error" : "Invalid OTP"})),
        ),
    }
}

#[derive(Deserialize)]
struct SignInSendOtpPayload {
    phone_number: String,
}

async fn sign_in_send_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignInSendOtpPayload>,
) -> impl IntoResponse {
    match repository::user::find_by_phone_number(ctx.db_conn.clone(), payload.phone_number.clone())
        .await
    {
        Some(user) => {
            if !user.is_verified {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "User not verified"})),
                );
            }
        }
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found"})),
            );
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
            (StatusCode::OK, Json(json!({"message": "OTP sent!"})))
        }
        Err(repository::otp::Error::OtpNotExpired) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "OTP not expired"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to send OTP"})),
        ),
    }
}

#[derive(Deserialize)]
struct SignInVerifyOtpPayload {
    phone_number: String,
    otp: String,
}

async fn sign_in_verify_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignInVerifyOtpPayload>,
) -> impl IntoResponse {
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
                                Ok(session) => (
                                    StatusCode::OK,
                                    Json(json!({
                                        "token": session.id,
                                    })),
                                ),
                                Err(_) => (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({
                                        "error": "Failed to create session"
                                    })),
                                ),
                            }
                        }
                        None => (
                            StatusCode::NOT_FOUND,
                            Json(json!({ "error": "User does not exist"})),
                        ),
                    }
                }
                _ => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Failed to verify OTP"})),
                ),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid OTP"})),
        ),
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
