use crate::repository;
use crate::types::Context;
use crate::utils::{self, otp};
use crate::utils::{notification, wallet};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use chrono::NaiveDate;
use futures::FutureExt;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::borrow::Cow;
use std::sync::Arc;
use validator::{Validate, ValidationError};

fn validate_phone_number(phone_number: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^\+234\d{10}$").expect("Invalid phone number regex");
    match regex.is_match(phone_number) {
        true => Ok(()),
        false => Err(
            ValidationError::new("INVALID_CLOSING_TIME").with_message(Cow::from(
                r"Phone number must be a nigerian phone number in international format (e.g: +234...)",
            )),
        ),
    }
}

#[derive(Deserialize, Validate)]
struct SignUpPayload {
    #[validate(email(code = "INVALID_USER_EMAIL", message = "Invalid email address"))]
    email: String,
    #[validate(custom(code = "INVALID_PHONE_NUMBER", function = "validate_phone_number"))]
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

    let mut tx = match ctx.db_conn.clone().pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to start transaction" })),
            );
        }
    };

    match repository::user::find_by_email_or_phone_number(
        &mut *tx,
        repository::user::FindByEmailOrPhoneNumber {
            email: payload.email.clone().to_lowercase(),
            phone_number: payload.phone_number.clone(),
        },
    )
    .await
    {
        Ok(None) => (),
        Ok(Some(user)) => {
            if user.email == payload.email {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "Email already in use" })),
                );
            }
            return (
                StatusCode::CONFLICT,
                Json(json!({ "error": "Phone number already in use" })),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch user" })),
            )
        }
    };

    let user = match repository::user::create(
        &mut *tx,
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
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Sign up failed!"
                })),
            )
        }
    };

    // TODO: Notification failing to send is insignificant for now
    let _ = tokio::spawn(notification::send(
        ctx.clone(),
        notification::Notification::registered(user.clone()),
        notification::Backend::Email,
    ));

    if let Err(_) = repository::wallet::create(&mut *tx, user.id.clone()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create wallet" })),
        );
    };

    match tx.commit().await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Sorry an error occurred" })),
            );
        }
    };

    // TODO: if an error occurs at this point the user can always request for another OTP
    let _ = tokio::spawn(otp::send(
        ctx.clone(),
        user,
        "auth.verification".to_string(),
    ));

    (
        StatusCode::OK,
        Json(json!({ "message" :"Check your phone for a verification OTP"})),
    )
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
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error" : "User not found"})),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch user" })),
            )
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
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found" })),
            )
        }
        Err(_) => {
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
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found"})),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch user" })),
            )
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

#[derive(Deserialize)]
struct RefreshTokensPayload {
    token: String,
}

async fn refresh_tokens(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<RefreshTokensPayload>,
) -> impl IntoResponse {
    match utils::auth::regenerate_tokens_for_session(ctx.clone(), payload.token).await {
        Ok(session) => (
            StatusCode::OK,
            Json(json!({
                "access_token": session.access_token,
                "refresh_token": session.refresh_token,
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to refresh tokens" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/sign-up/strategy/credentials", post(sign_up))
        .route("/sign-in/strategy/phone", post(sign_in))
        .route("/verification/send-otp", post(send_otp))
        .route("/verification/verify-otp", post(verify_otp))
        .route("/refresh", post(refresh_tokens))
}
