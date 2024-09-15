use crate::repository;
use crate::types::Context;
use crate::utils;
use crate::utils::notification;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
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

#[derive(Serialize, Deserialize)]
struct SendPasswordResetEmailPayload {
    email: String,
}

async fn send_password_reset_email(
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<SendPasswordResetEmailPayload>,
) -> impl IntoResponse {
    let user = match repository::user::find_by_email(ctx.db_conn.clone(), payload.email).await {
        Some(user) => user,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "A user with the email address does not exist" })),
            )
        } // Err(_) => {
          //     return (
          //         StatusCode::INTERNAL_SERVER_ERROR,
          //         Json(json!({ "error": "Failed to fetch user" })),
          //     )
          // }
    };

    if user.is_verified == false {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "User not verified"
            })),
        );
    }

    let expires_at = Utc::now().naive_utc() + chrono::Duration::minutes(5);
    let code = {
        let data = format!("{}-{}", user.id, expires_at);
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        base16ct::lower::encode_string(&hash)
    };

    let password_reset =
        match repository::password_reset::find_by_user_id(ctx.db_conn.clone(), user.id.clone())
            .await
        {
            Ok(Some(pr)) => {
                match repository::password_reset::update_by_id(
                    ctx.db_conn.clone(),
                    pr.id,
                    repository::password_reset::UpdatePasswordResetPayload { code },
                )
                .await
                {
                    Ok(pr) => pr,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": "Failed to create password reset entry" })),
                        );
                    }
                }
            }
            Ok(None) => {
                match repository::password_reset::create(
                    ctx.db_conn.clone(),
                    repository::password_reset::CreatePasswordResetPayload {
                        code,
                        expires_at,
                        user_id: user.id.clone(),
                    },
                )
                .await
                {
                    Ok(pr) => pr,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": "Failed to create password reset entry" })),
                        );
                    }
                }
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create password reset entry" })),
                );
            }
        };

    match notification::send(
        ctx.clone(),
        notification::Notification::password_reset_requested(user, password_reset),
        notification::Backend::Email,
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Check your email for a password reset link " })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to send password reset email" })),
        ),
    }
}

async fn reset_password(
    State(ctx): State<Arc<Context>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let password_reset =
        match repository::password_reset::find_by_code(ctx.db_conn.clone(), code).await {
            Ok(Some(pr)) => pr,
            Ok(None) => {
                (return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid request" })),
                ))
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Server error occurred" })),
                )
            }
        };

    if Utc::now().naive_utc() > password_reset.expires_at {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid request" })),
        );
    }

    unimplemented!();

    (
        StatusCode::OK,
        Json(json!({ "message": "Password reset successful!" })),
    )
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
        .route("/reset-password", post(send_password_reset_email))
        .route("/reset-password/:code", get(reset_password))
}
