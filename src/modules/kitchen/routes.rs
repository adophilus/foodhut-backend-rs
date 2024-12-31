use std::{borrow::Cow, sync::Arc};

use super::repository;
use crate::modules::{auth::middleware::AdminAuth, user};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::io::Read;
use tempfile::NamedTempFile;
use validator::{Validate, ValidationError};

use crate::{
    modules::{auth::middleware::Auth, storage},
    types::Context,
    utils::{self, pagination::Pagination},
};

const KITCHEN_TYPES: [&str; 4] = ["Chinese", "Cuisine", "Fast Food", "Local"];

#[derive(Deserialize, Validate)]
struct CreateKitchenPayload {
    pub name: String,
    pub address: String,
    pub phone_number: String,
    #[validate(custom(function = "validate_kitchen_type"))]
    #[serde(rename = "type")]
    pub type_: String,
    #[validate(custom(code = "INVALID_OPENING_TIME", function = "validate_opening_time"))]
    pub opening_time: String,
    #[validate(custom(code = "INVALID_CLOSING_TIME", function = "validate_closing_time"))]
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
    pub city_id: String,
}

fn validate_kitchen_type(type_: &str) -> Result<(), ValidationError> {
    match KITCHEN_TYPES.contains(&type_) {
        true => Ok(()),
        false => Err(ValidationError::new("INVALID_KITCHEN_TYPE")
            .with_message(Cow::from("Invalid kitchen type"))),
    }
}

fn validate_opening_time(time_str: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid opening time regex");
    match regex.is_match(time_str) {
        true => Ok(()),
        false => Err(
            ValidationError::new("INVALID_OPENING_TIME").with_message(Cow::from(
                r"Opening time must be in 24 hour format (e.g: 08:00)",
            )),
        ),
    }
}

fn validate_closing_time(time_str: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid closing time regex");
    match regex.is_match(time_str) {
        true => Ok(()),
        false => Err(
            ValidationError::new("INVALID_CLOSING_TIME").with_message(Cow::from(
                r"Closing time must be in 24 hour format (e.g: 20:00)",
            )),
        ),
    }
}

async fn create_kitchen(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateKitchenPayload>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return utils::validation::into_response(errors);
    }

    match repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone()).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to find kitchen"})),
            )
        }
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "You already have a kitchen"})),
            )
        }
        Ok(None) => (),
    };

    if let Err(_) = repository::create(
        &ctx.db_conn.pool,
        repository::CreateKitchenPayload {
            name: payload.name,
            address: payload.address,
            r#type: payload.type_,
            phone_number: payload.phone_number,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            city_id: payload.city_id,
            owner_id: auth.user.id.clone(),
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Kitchen creation failed"})),
        );
    }

    if let Err(_) = user::repository::update_by_id(
        &ctx.db_conn.pool,
        auth.user.id.clone(),
        user::repository::UpdateUserPayload {
            has_kitchen: Some(true),
            email: None,
            birthday: None,
            last_name: None,
            first_name: None,
            phone_number: None,
            profile_picture: None,
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Kitchen creation failed"})),
        );
    };

    (
        StatusCode::CREATED,
        Json(json!({ "message": "Kitchen created!"})),
    )
}

async fn get_kitchens(
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<repository::FindManyFilters>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::find_many(&ctx.db_conn.pool, pagination.clone(), filters).await {
        Ok(paginated_kitchens) => (StatusCode::OK, Json(json!(paginated_kitchens))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchens"})),
        ),
    }
}

async fn get_kitchen_by_profile(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id).await {
        Ok(Some(kitchen)) => (StatusCode::OK, Json(json!(kitchen))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Kitchen not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchen"})),
        ),
    }
}

async fn get_kitchen_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(kitchen)) => (StatusCode::OK, Json(json!(kitchen))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Kitchen not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchens"})),
        ),
    }
}

async fn fetch_kitchen_types() -> impl IntoResponse {
    Json(json!(KITCHEN_TYPES))
}

async fn fetch_kitchen_cities(
    State(ctx): State<Arc<Context>>
) -> impl IntoResponse {
    match repository::find_many_cities(&ctx.db_conn.pool).await {
        Ok(cities) => Json(json!(cities)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch cities"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateKitchenPayload {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone_number: Option<String>,
    #[validate(custom(function = "validate_kitchen_type"))]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[validate(custom(function = "validate_opening_time"))]
    pub opening_time: Option<String>,
    #[validate(custom(function = "validate_closing_time"))]
    pub closing_time: Option<String>,
    pub preparation_time: Option<String>,
    pub delivery_time: Option<String>,
    pub is_available: Option<bool>,
}

async fn update_kitchen_by_profile(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<UpdateKitchenPayload>,
) -> Response {
    match repository::find_by_owner_id(&ctx.db_conn.pool, auth.clone().user.id).await {
        Ok(Some(kitchen)) => update_kitchen_by_id(
            Path { 0: kitchen.id },
            State(ctx),
            auth.clone(),
            Json(payload),
        )
        .await
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Kitchen not found" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch kitchen" })),
        )
            .into_response(),
    }
}

async fn update_kitchen_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<UpdateKitchenPayload>,
) -> impl IntoResponse {
    let kitchen = match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to find kitchen"})),
            )
        }
        Ok(Some(kitchen)) => kitchen,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Kitchen not found"})),
            )
        }
    };

    if !repository::is_owner(&auth.user, &kitchen) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this kitchen"})),
        );
    }

    match repository::update_by_id(
        &ctx.db_conn.pool,
        kitchen.id,
        repository::UpdateKitchenPayload {
            name: payload.name,
            address: payload.address,
            phone_number: payload.phone_number,
            r#type: payload.type_,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            cover_image: None,
            rating: None,
            likes: None,
            is_available: payload.is_available,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Kitchen updated successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to update kitchen" })),
        ),
    }
}

async fn like_kitchen_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    match repository::like_by_id(&ctx.db_conn.pool, id, auth.user.id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Kitchen liked successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to like kitchen" })),
        ),
    }
}

async fn unlike_kitchen_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    match repository::unlike_by_id(&ctx.db_conn.pool, id, auth.user.id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Kitchen unliked successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to unlike kitchen" })),
        ),
    }
}

#[derive(TryFromMultipart)]
struct SetKitchenCoverImage {
    cover_image: FieldData<NamedTempFile>,
}

async fn set_kitchen_cover_image_by_profile(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    payload: TypedMultipart<SetKitchenCoverImage>,
) -> impl IntoResponse {
    let kitchen = match repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone()).await
    {
        Ok(Some(kitchen)) => kitchen,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Kitchen not found" })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to find kitchen" })),
            )
                .into_response()
        }
    };
    return set_kitchen_cover_image(State(ctx), auth, kitchen, payload)
        .await
        .into_response();
}

async fn set_kitchen_cover_image_by_id(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
    payload: TypedMultipart<SetKitchenCoverImage>,
) -> impl IntoResponse {
    let kitchen = match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(kitchen)) => kitchen,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Kitchen not found" })),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to find kitchen" })),
            )
                .into_response()
        }
    };
    return set_kitchen_cover_image(State(ctx), auth, kitchen, payload)
        .await
        .into_response();
}

async fn set_kitchen_cover_image(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    kitchen: repository::Kitchen,
    TypedMultipart(mut payload): TypedMultipart<SetKitchenCoverImage>,
) -> impl IntoResponse {
    if !repository::is_owner(&auth.user, &kitchen) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "You are not the owner of this kitchen" })),
        );
    }

    let mut buf: Vec<u8> = vec![];

    if let Err(err) = payload.cover_image.contents.read_to_end(&mut buf) {
        tracing::error!("Failed to read the uploaded file {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload image" })),
        );
    };

    let profile_picture = match kitchen.cover_image.0 {
        Some(cover_image) => storage::update_file(ctx.storage.clone(), cover_image, buf).await,
        None => storage::upload_file(ctx.storage.clone(), buf).await,
    };

    let cover_image = match profile_picture {
        Ok(cover_image) => cover_image,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to upload image" })),
            )
        }
    };

    if let Err(_) = repository::update_by_id(
        &ctx.db_conn.pool,
        kitchen.id,
        repository::UpdateKitchenPayload {
            name: None,
            address: None,
            phone_number: None,
            r#type: None,
            opening_time: None,
            closing_time: None,
            preparation_time: None,
            delivery_time: None,
            cover_image: Some(cover_image),
            rating: None,
            likes: None,
            is_available: None,
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

async fn block_kitchen_by_id(_: AdminAuth) -> impl IntoResponse {
    // TODO: this needs to be implemented
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Not implemented yet!" })),
    )
}

async fn unblock_kitchen_by_id(_: AdminAuth) -> impl IntoResponse {
    // TODO: this needs to be implemented
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Not implemented yet!" })),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_kitchen).get(get_kitchens))
        .route(
            "/profile",
            get(get_kitchen_by_profile).patch(update_kitchen_by_profile),
        )
        .route(
            "/profile/profile-picture",
            put(set_kitchen_cover_image_by_profile),
        )
        .route("/:id", get(get_kitchen_by_id).patch(update_kitchen_by_id))
        .route("/:id/profile-picture", put(set_kitchen_cover_image_by_id))
        .route("/:id/like", put(like_kitchen_by_id))
        .route("/:id/unlike", put(unlike_kitchen_by_id))
        .route("/:id/block", put(block_kitchen_by_id))
        .route("/:id/unblock", put(unblock_kitchen_by_id))
        .route("/types", get(fetch_kitchen_types))
        .route("/cities", get(fetch_kitchen_cities))
}
