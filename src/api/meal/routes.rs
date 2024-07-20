use std::{borrow::Cow, io::Read, sync::Arc};

use axum::{
    async_trait,
    body::Bytes,
    extract::{multipart::Field, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use axum_typed_multipart::{
    FieldData, TryFromField, TryFromMultipart, TypedMultipart, TypedMultipartError,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use num_bigint::{BigInt, Sign};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tempfile::NamedTempFile;
use ulid::Ulid;
use validator::{Validate, ValidationError};

use crate::{
    api::auth::middleware::Auth,
    repository::{self, user::User},
    types::Context,
    utils::{self, pagination::Pagination},
};

#[derive(Debug, Clone)]
struct Price(BigDecimal);

#[async_trait]
impl TryFromField for Price {
    async fn try_from_field<'a>(
        field: Field<'a>,
        limit: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        field
            .text()
            .await
            .map(|text| {
                text.parse::<f32>().map(|price| {
                    Price(BigDecimal::from_f32(price).unwrap_or(BigDecimal::from_u8(0).unwrap()))
                })
            })
            .map_err(|err| {
                tracing::debug!("Error occurred while parsing body: {}", err);
                TypedMultipartError::InvalidRequestBody { source: err }
            })
            .unwrap()
            .map_err(|err| {
                tracing::debug!("Error occurred while parsing body: {}", err);
                TypedMultipartError::UnknownField {
                    field_name: String::from("price"),
                }
            })
    }
}

#[derive(TryFromMultipart)]
struct CreateMealPayload {
    name: String,
    description: String,
    price: Price,
    #[form_data(limit = "10MiB")]
    cover_image: FieldData<NamedTempFile>,
}

async fn create_meal(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    TypedMultipart(mut payload): TypedMultipart<CreateMealPayload>,
) -> impl IntoResponse {
    let kitchen =
        match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
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

    let mut buf: Vec<u8> = vec![];

    if let Err(err) = payload.cover_image.contents.read_to_end(&mut buf) {
        tracing::debug!("Failed to read the uploaded file {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload image" })),
        );
    }

    let cover_image = match utils::storage::upload_file(ctx.storage.clone(), buf).await {
        Ok(url) => url,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to upload image" })),
            );
        }
    };

    match repository::meal::create(
        ctx.db_conn.clone(),
        repository::meal::CreateMealPayload {
            name: payload.name,
            description: payload.description,
            price: payload.price.0,
            cover_image,
            kitchen_id: kitchen.id,
        },
    )
    .await
    {
        Ok(meal) => (
            StatusCode::CREATED,
            Json(json!({
                "message": "Meal created!",
                "id": meal.id
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Meal creation failed"})),
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

async fn get_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::meal::find_by_id(ctx.db_conn.clone(), id).await {
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
}

async fn update_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<UpdateMealPayload>,
) -> impl IntoResponse {
    let kitchen = match repository::kitchen::find_by_owner_id(
        ctx.db_conn.clone(),
        auth.user.clone().id,
    )
    .await
    {
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

    let meal = match repository::meal::find_by_id(ctx.db_conn.clone(), id.clone()).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to find meal"})),
            )
        }
        Ok(Some(kitchen)) => kitchen,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Meal not found"})),
            )
        }
    };

    if !repository::meal::is_owner(auth.user.clone(), kitchen, meal) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this meal"})),
        );
    }

    match repository::meal::update_by_id(
        ctx.db_conn.clone(),
        id,
        repository::meal::UpdateMealPayload {
            name: payload.name,
            description: payload.description,
            price: payload.price,
            rating: None,
            is_available: None,
            cover_image: None,
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

async fn delete_meal_by_id(
    Path(id): Path<String>,
    auth: Auth,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::meal::find_by_id(ctx.db_conn.clone(), id.clone()).await {
        Ok(maybe_meal) => {
            if maybe_meal.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Meal not found"})),
                );
            }

            let meal = maybe_meal.unwrap();
            let kitchen = if let Ok(Some(kitchen)) =
                repository::kitchen::find_by_id(ctx.db_conn.clone(), meal.kitchen_id.clone()).await
            {
                kitchen
            } else {
                return (
                    StatusCode::OK,
                    Json(json!({ "error": "Kitchen not found" })),
                );
            };

            if !repository::meal::is_owner(auth.user.clone(), kitchen, meal.clone()) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "You are not the owner of this kitchen"})),
                );
            }

            if let Err(_) =
                utils::storage::delete_file(ctx.storage.clone(), meal.cover_image.clone()).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete meal" })),
                );
            }

            match repository::meal::delete_by_id(ctx.db_conn.clone(), id.clone()).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal deleted successfully" })),
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to delete meal" })),
                ),
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to delete meal" })),
        ),
    }
}

async fn like_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    match repository::meal::like_by_id(ctx.db_conn.clone(), id, auth.user.id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Meal liked successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to like meal" })),
        ),
    }
}

async fn unlike_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    match repository::meal::unlike_by_id(ctx.db_conn.clone(), id, auth.user.id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Meal unliked successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to unlike meal" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_meal).get(get_meals))
        .route(
            "/:id",
            get(get_meal_by_id)
                .patch(update_meal_by_id)
                .delete(delete_meal_by_id),
        )
        .route("/:id/like", put(like_meal_by_id))
        .route("/:id/unlike", put(unlike_meal_by_id))
}
