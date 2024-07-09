use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationError};

use crate::{
    api::auth::middleware::Auth,
    repository,
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
    #[validate(custom(function = "validate_opening_time"))]
    pub opening_time: String,
    #[validate(custom(function = "validate_closing_time"))]
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
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

    match repository::kitchen::create(
        ctx.db_conn.clone(),
        repository::kitchen::CreateKitchenPayload {
            name: payload.name,
            address: payload.address,
            type_: payload.type_,
            phone_number: payload.phone_number,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            owner_id: auth.user.id,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Kitchen created!"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Kitchen creation failed"})),
        ),
    }
}

async fn get_kitchens(
    State(ctx): State<Arc<Context>>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::kitchen::find_many(ctx.db_conn.clone(), pagination.clone()).await {
        Ok(paginated_kitchens) => (StatusCode::OK, Json(json!(paginated_kitchens))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchens"})),
        ),
    }
}

async fn get_kitchen_by_profile(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
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
    match repository::kitchen::find_by_id(ctx.db_conn.clone(), id).await {
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

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_kitchen).get(get_kitchens))
        .route("/profile", get(get_kitchen_by_profile))
        .route("/:id", get(get_kitchen_by_id))
        .route("/types", get(fetch_kitchen_types))
}
