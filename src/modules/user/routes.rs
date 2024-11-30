use super::repository;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use std::io::Read;
use std::sync::Arc;
use tempfile::NamedTempFile;

use crate::{
    modules::auth::middleware::Auth,
    types::Context,
    utils::{database::DatabaseConnection, storage},
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
    match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(json!(user))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User not found"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch user" })),
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
    let update_payload = repository::UpdateUserPayload {
        email: payload.email,
        phone_number: payload.phone_number,
        first_name: payload.first_name,
        last_name: payload.last_name,
        birthday: payload.birthday,
        has_kitchen: None,
        profile_picture: None,
    };

    match repository::update_by_id(&db_conn.pool, user_id, update_payload).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Update successful"})),
        )
            .into_response(),
        Err(repository::Error::UnexpectedError) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Update failed" })),
        )
            .into_response(),
    }
}

#[derive(TryFromMultipart)]
struct SetUserProfilePicture {
    profile_picture: FieldData<NamedTempFile>,
}

async fn set_user_profile_picture_by_id(
    auth: Auth,
    state: State<Arc<Context>>,
    Path(id): Path<String>,
    payload: TypedMultipart<SetUserProfilePicture>,
) -> impl IntoResponse {
    if auth.user.id != id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You are not allowed to update this profile picture" })),
        )
            .into_response();
    }

    return set_user_profile_picture(state, auth, payload)
        .await
        .into_response();
}

async fn set_user_profile_picture(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    TypedMultipart(mut payload): TypedMultipart<SetUserProfilePicture>,
) -> impl IntoResponse {
    let mut buf: Vec<u8> = vec![];

    if let Err(err) = payload.profile_picture.contents.read_to_end(&mut buf) {
        tracing::error!("Failed to read the uploaded file {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload image" })),
        );
    };

    let profile_picture = match auth.user.profile_picture.0 {
        Some(profile_picture) => {
            storage::update_file(ctx.storage.clone(), profile_picture, buf).await
        }
        None => storage::upload_file(ctx.storage.clone(), buf).await,
    };

    let profile_picture = match profile_picture {
        Ok(profile_picture) => profile_picture,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to upload image" })),
            )
        }
    };

    if let Err(_) = repository::update_by_id(
        &ctx.db_conn.pool,
        auth.user.id,
        repository::UpdateUserPayload {
            email: None,
            phone_number: None,
            first_name: None,
            last_name: None,
            birthday: None,
            has_kitchen: None,
            profile_picture: Some(profile_picture),
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to update profile picture" })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({ "message": "Profile picture updated successfully" })),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route(
            "/profile",
            get(get_user_by_profile).patch(update_user_by_profile),
        )
        .route("/profile/profile-picture", put(set_user_profile_picture))
        .route("/:id", get(get_user_by_id).patch(update_user_by_id))
        .route("/:id/profile-picture", put(set_user_profile_picture_by_id))
}
