use axum::{extract::{Path, State}, response::IntoResponse, routing::{get, patch}, extract::Json, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    api::auth::middleware::Auth,
    repository::{self, user::User},
    types::{ApiResponse, Context},
};

async fn profile(auth: Auth) -> ApiResponse<User, &'static str> {
    ApiResponse::ok(auth.user)
}

#[derive(Deserialize)]
struct UpdateUserPayload {
    email: Option<String>,
    phone_number: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    birthday: Option<NaiveDate>,
    // profile_picture: Option
}

async fn get_user_by_id(State(ctx): State<Arc<Context>>, Path(id): Path<String>) -> ApiResponse<User, &'static str> {
    match repository::user::find_by_id(ctx.db_conn.clone(), id)
    .await {
            Some(user) => ApiResponse::ok(user),
            None => ApiResponse::err("User not found"),
        }
}

async fn update_user_by_id(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserPayload>,
) -> ApiResponse<&'static str, &'static str>{
    tracing::debug!(auth.user.id);
    tracing::debug!(id);
    if auth.user.id != id {
        return ApiResponse::err("Access denied");
    }

    let update_payload = repository::user::UpdateUserPayload {
        email: payload.email,
        phone_number: payload.phone_number,
        first_name: payload.first_name,
        last_name: payload.last_name,
        birthday: payload.birthday,
        profile_picture_url: None
    };

    match repository::user::update_by_id(ctx.db_conn.clone(), id, update_payload).await {
        Ok(_) => ApiResponse::ok("Update successful"),
        Err(repository::user::Error::UnexpectedError) => ApiResponse::err("Update failed"),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/profile", get(profile))
        .route("/:id", get(get_user_by_id).patch(update_user_by_id))
}
