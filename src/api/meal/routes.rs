use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bigdecimal::BigDecimal;
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

#[derive(Deserialize, Validate)]
struct CreateMealPayload {
    pub name: String,
    pub description: String,
    pub price: BigDecimal,
    pub tags: Vec<String>,
}

async fn create_meal(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateMealPayload>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return utils::validation::into_response(errors);
    }

    match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
        Ok(Some(kitchen)) => match repository::meal::create(
            ctx.db_conn.clone(),
            repository::meal::CreateMealPayload {
                name: payload.name,
                description: payload.description,
                price: payload.price,
                cover_image_url: "".to_string(),
                tags: payload.tags,
                kitchen_id: kitchen.id,
            },
        )
        .await
        {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({ "message": "Meal created!"})),
            ),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Meal creation failed"})),
            ),
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Kitchen not found"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create meal"})),
        ),
    }
}

async fn get_meals(State(ctx): State<Arc<Context>>, pagination: Pagination) -> impl IntoResponse {
    match repository::meal::find_many(ctx.db_conn.clone(), pagination.clone()).await {
        Ok(paginated_meals) => (StatusCode::OK, Json(json!(paginated_meals))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch meals"})),
        ),
    }
}

struct MealPath {
    pub meal_id: String,
}

async fn get_meal_by_id(
    Path(path): Path<MealPath>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::meal::find_by_id(ctx.db_conn.clone(), path.meal_id).await {
        Ok(Some(meal)) => (StatusCode::OK, Json(json!(meal))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Meal not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch meals"})),
        ),
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateMealPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<BigDecimal>,
    pub tags: Option<Vec<String>>,
}

async fn update_meal_by_id(
    Path(path): Path<MealPath>,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<UpdateMealPayload>,
) -> impl IntoResponse {
    match repository::meal::update_by_id(
        ctx.db_conn.clone(),
        path.meal_id,
        repository::meal::UpdateMealPayload {
            name: payload.name,
            rating: None,
            is_available: None,
            description: payload.description,
            price: payload.price,
            cover_image_url: None,
            tags: payload.tags,
            kitchen_id: None,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Meal updated successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to update meal" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_meal).get(get_meals))
        .route("/:meal_id", get(get_meal_by_id).patch(update_meal_by_id))
}
