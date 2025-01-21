use crate::{
    modules::{
        auth::{self, middleware::Auth},
        notification, user,
    },
    Context,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
struct SignInPayload {
    phone_number: String,
}

async fn sign_in(
    State(ctx): State<Arc<Context>>,
    payload: Json<SignInPayload>,
) -> impl IntoResponse {
    let user =
        user::repository::find_by_phone_number(&ctx.db_conn.pool, payload.phone_number.clone())
            .await
            .unwrap()
            .unwrap();
    let session = auth::service::auth::create_session(ctx, user.id)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(
            json!({ "access_token": session.access_token, "refresh_token": session.refresh_token }),
        ),
    )
}

async fn send_test_email(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match notification::service::send(
        ctx,
        notification::service::Notification::Registered(notification::service::types::Registered {
            user: auth.user,
        }),
        notification::service::Backend::Email,
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Email sent"}))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to send email" })),
        ),
    }
}

async fn send_test_push_notification(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    match notification::service::send(
        ctx,
        notification::service::Notification::Registered(notification::service::types::Registered {
            user: auth.user,
        }),
        notification::service::Backend::Push,
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Notification sent" })),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to send notification" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/auth/sign-in", post(sign_in))
        .route("/test/email", post(send_test_email))
        .route("/test/push-notification", post(send_test_push_notification))
}
