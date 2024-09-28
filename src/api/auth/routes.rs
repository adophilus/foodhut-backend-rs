use crate::repository;
use crate::types::Context;
use crate::utils::notification;
use crate::utils::{self, otp};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
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
                notification::Notification::registered(user.clone()),
                notification::Backend::Email,
            )
            .await;

            match otp::send(ctx.clone(), user, "auth.verification".to_string()).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({ "message" :"Check your phone for a verification OTP"})),
                ),
                Err(otp::SendError::NotExpired) => unreachable!(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error" : "Failed to send OTP"})),
                ),
            }
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

async fn send_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SendVerificationOtpPayload>,
) -> impl IntoResponse {
    let user = match repository::user::find_by_phone_number(
        ctx.db_conn.clone(),
        payload.phone_number.clone(),
    )
    .await
    {
        Some(user) => user,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error" : "User not found"})),
            );
        }
    };

    if user.is_verified {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error" : "User already verified"})),
        );
    }

    match otp::send(ctx.clone(), user, "auth.verification".to_string()).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message" :"Check your phone for a verification OTP" })),
        ),
        Err(otp::SendError::NotExpired) => (
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

async fn verify_otp(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<VerifyOtpPayload>,
) -> impl IntoResponse {
    let user = match repository::user::find_by_phone_number(
        ctx.db_conn.clone(),
        payload.phone_number.clone(),
    )
    .await
    {
        Some(user) => user,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch user account" })),
            )
        }
    };

    match otp::verify(
        ctx.clone(),
        user.clone(),
        "auth.verification".to_string(),
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
                Ok(_) => {
                    let session = match utils::auth::create_session(ctx.clone(), user.id).await {
                        Ok(session) => session,
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "error": "Failed to create session" })),
                            )
                        }
                    };

                    (
                        StatusCode::OK,
                        Json(
                            json!({ "access_token": session.access_token, "refresh_token": session.refresh_token }),
                        ),
                    )
                }
                _ => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error" : "Failed to verify OTP"})),
                ),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error" : "Invalid or expired OTP"})),
        ),
    }
}

#[derive(Deserialize)]
struct SignInSendOtpPayload {
    phone_number: String,
}

async fn sign_in(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SignInSendOtpPayload>,
) -> impl IntoResponse {
    let user = match repository::user::find_by_phone_number(
        ctx.db_conn.clone(),
        payload.phone_number.clone(),
    )
    .await
    {
        Some(user) => user,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found"})),
            );
        }
    };

    if !user.is_verified {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "User not verified"})),
        );
    }

    match otp::send(ctx.clone(), user, "auth.verification".to_string()).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Check your phone for a verification OTP"})),
        ),
        Err(otp::SendError::NotExpired) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "OTP not expired"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to send OTP"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/sign-up/strategy/credentials", post(sign_up))
        .route("/sign-in/strategy/phone", post(sign_in))
        .route("/verification/send-otp", post(send_otp))
        .route("/verification/verify-otp", post(verify_otp))
}
