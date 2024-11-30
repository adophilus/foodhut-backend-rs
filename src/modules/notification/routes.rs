use std::sync::Arc;

use super::repository;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct CreatePushTokenPayload {
    token: String,
}

async fn create_push_token(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreatePushTokenPayload>,
) -> impl IntoResponse {
    match repository::push_token::create(
        &ctx.db_conn.pool,
        repository::push_token::CreatePushTokenPayload {
            token: payload.token,
            user_id: auth.user.id,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "message": "Push token created created!",
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Push token creation failed"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/push-token", post(create_push_token))
}
