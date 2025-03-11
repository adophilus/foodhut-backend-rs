use crate::{
    modules::{
        auth::{self, middleware::Auth},
        notification, user, zoho,
    },
    Context,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
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

    let session = auth::service::auth::create_session(&ctx.db_conn.pool, user.id)
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
        notification::service::Notification::registered(auth.user),
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
        notification::service::Notification::registered(auth.user),
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

async fn generate_zoho_tokens(State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match zoho::service::generate_token_link(ctx).await {
        Ok(url) => Redirect::to(&url).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Sorry an error occurred" })),
        )
            .into_response(),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/auth/sign-in", post(sign_in))
        .route("/test/email", post(send_test_email))
        .route("/test/push-notification", post(send_test_push_notification))
        .route("/zoho/generate-token", get(generate_zoho_tokens))
}
