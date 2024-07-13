use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::{
    api::auth::middleware::Auth,
    repository::{self, user::User},
    types::Context,
    utils::database::DatabaseConnection,
};

async fn get_user_by_profile(auth: Auth) -> impl IntoResponse {
    (StatusCode::OK, Json(auth.user))
}

#[derive(Deserialize)]
struct UpdateUserPayload {
    email: Option<String>,
    phone_number: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    birthday: Option<NaiveDate>,
}

async fn get_user_by_id(
    State(ctx): State<Arc<Context>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repository::user::find_by_id(ctx.db_conn.clone(), id).await {
        Some(user) => (StatusCode::OK, Json(json!(user))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found"})),
        ),
    }
}

async fn update_user_by_id(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserPayload>,
) -> Response {
    if auth.user.id != id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Access denied"})),
        )
            .into_response();
    }

    update_user_profile(ctx.db_conn.clone(), id, payload).await
}

async fn update_user_by_profile(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<UpdateUserPayload>,
) -> impl IntoResponse {
    update_user_profile(ctx.db_conn.clone(), auth.user.id, payload).await
}

async fn update_user_profile(
    db_conn: DatabaseConnection,
    user_id: String,
    payload: UpdateUserPayload,
) -> Response {
    let update_payload = repository::user::UpdateUserPayload {
        email: payload.email,
        phone_number: payload.phone_number,
        first_name: payload.first_name,
        last_name: payload.last_name,
        birthday: payload.birthday,
        has_kitchen: None,
        profile_picture_url: None,
    };

    match repository::user::update_by_id(db_conn.clone(), user_id, update_payload).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Update successful"})),
        )
            .into_response(),
        Err(repository::user::Error::UnexpectedError) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Update failed" })),
        )
            .into_response(),
    }
}

async fn set_user_profile_picture() {
    todo!("Not implemented yet!")
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route(
            "/profile",
            get(get_user_by_profile).patch(update_user_by_profile),
        )
        .route("/:id", get(get_user_by_id).patch(update_user_by_id))
        .route("/:id/profile-picture", put(set_user_profile_picture))
}
