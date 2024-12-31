use std::{io::Read, sync::Arc};

use super::repository::{self, MealWithCartStatus};
use crate::{
    modules::{
        auth::middleware::Auth,
        cart::{self, repository::CreateCartPayload},
        kitchen, storage, user,
    },
    utils::pagination,
};
use axum::{
    async_trait,
    extract::{multipart::Field, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use axum_typed_multipart::{
    FieldData, TryFromField, TryFromMultipart, TypedMultipart, TypedMultipartError,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use serde::Deserialize;
use serde_json::json;
use tempfile::NamedTempFile;

use crate::{types::Context, utils::pagination::Pagination};

#[derive(Debug, Clone)]
pub struct Price(BigDecimal);

#[async_trait]
impl TryFromField for Price {
    async fn try_from_field<'a>(
        field: Field<'a>,
        _: Option<usize>,
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
                tracing::error!("Error occurred while parsing body: {}", err);
                TypedMultipartError::InvalidRequestBody { source: err }
            })
            .unwrap()
            .map_err(|err| {
                tracing::error!("Error occurred while parsing body: {}", err);
                TypedMultipartError::UnknownField {
                    field_name: String::from("price"),
                }
            })
    }
}

#[derive(TryFromMultipart)]
pub struct CreateMealPayload {
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
    let kitchen = match kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id).await
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

    let mut buf: Vec<u8> = vec![];

    if let Err(err) = payload.cover_image.contents.read_to_end(&mut buf) {
        tracing::error!("Failed to read the uploaded file {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload image" })),
        );
    }

    let cover_image = match storage::upload_file(ctx.storage.clone(), buf).await {
        Ok(media) => media,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to upload image" })),
            );
        }
    };

    match repository::create(
        &ctx.db_conn.pool,
        repository::CreateMealPayload {
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

#[derive(Deserialize)]
struct Filters {
    pub kitchen_id: Option<String>,
    pub search: Option<String>,
    pub is_liked: Option<bool>,
    pub as_kitchen: Option<bool>,
}

async fn get_meals(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    pagination: Pagination,
    Query(filters): Query<Filters>,
) -> impl IntoResponse {
    let is_liked_by = match filters.is_liked {
        Some(true) => Some(auth.user.id.clone()),
        _ => None,
    };

    let cart = match cart::repository::find_active_cart_by_owner_id(
        &ctx.db_conn.pool,
        auth.user.id.clone(),
    )
    .await
    {
        Ok(Some(cart)) => cart,
        Ok(None) => {
            match cart::repository::create(
                &ctx.db_conn.pool,
                cart::repository::CreateCartPayload {
                    owner_id: auth.user.id.clone(),
                },
            )
            .await
            {
                Ok(cart) => cart,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to fetch meals"})),
                    )
                }
            }
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch meals"})),
            )
        }
    };

    let meals_result = if user::repository::is_admin(&auth.user) {
        repository::find_many_as_admin(
            &ctx.db_conn.pool,
            pagination.clone(),
            repository::FindManyAsAdminFilters {
                kitchen_id: filters.kitchen_id,
                search: filters.search,
                is_liked_by,
            },
        )
        .await
    }
    else if filters.as_kitchen.is_some() {
        repository::find_many_as_kitchen(
            &ctx.db_conn.pool,
            pagination.clone(),
            repository::FindManyAsKitchenFilters {
                kitchen_id: filters.kitchen_id,
                search: filters.search,
                is_liked_by,
                owner_id: auth.user.id.clone()
            },
        )
        .await
    }
    else {
        repository::find_many_as_user(
            &ctx.db_conn.pool,
            pagination.clone(),
            repository::FindManyAsUserFilters {
                kitchen_id: filters.kitchen_id,
                search: filters.search,
                is_liked_by,
            },
        )
        .await
    };

    match meals_result {
        Ok(paginated_meals) => {
            let augmented_meals = paginated_meals
                .items
                .clone()
                .into_iter()
                .map(|meal| MealWithCartStatus {
                    id: meal.id.clone(),
                    name: meal.name,
                    description: meal.description,
                    rating: meal.rating,
                    original_price: meal.original_price,
                    price: meal.price,
                    likes: meal.likes,
                    cover_image: meal.cover_image,
                    is_available: meal.is_available,
                    in_cart: cart
                        .items
                        .0
                        .iter()
                        .find(|item| item.meal_id == meal.id)
                        .is_some(),
                    kitchen_id: meal.kitchen_id,
                    created_at: meal.created_at,
                    updated_at: meal.updated_at,
                })
                .collect::<Vec<_>>();

            (
                StatusCode::OK,
                Json(json!(pagination::Paginated::new(
                    augmented_meals,
                    paginated_meals.meta.total,
                    paginated_meals.meta.page,
                    paginated_meals.meta.per_page,
                ))),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch meals"})),
        ),
    }
}

async fn get_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    let cart = match cart::repository::find_active_cart_by_owner_id(
        &ctx.db_conn.pool,
        auth.user.id.clone(),
    )
    .await
    {
        Ok(Some(cart)) => cart,
        Ok(None) => {
            match cart::repository::create(
                &ctx.db_conn.pool,
                CreateCartPayload {
                    owner_id: auth.user.id.clone(),
                },
            )
            .await
            {
                Ok(cart) => cart,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to fetch meals"})),
                    )
                }
            }
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch meals"})),
            )
        }
    };

    match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(meal)) => {
            let augmented_meal = MealWithCartStatus {
                id: meal.id.clone(),
                name: meal.name,
                description: meal.description,
                rating: meal.rating,
                original_price: meal.original_price,
                price: meal.price,
                likes: meal.likes,
                cover_image: meal.cover_image,
                is_available: meal.is_available,
                in_cart: cart
                    .items
                    .0
                    .iter()
                    .find(|item| item.meal_id == meal.id)
                    .is_some(),
                kitchen_id: meal.kitchen_id,
                created_at: meal.created_at,
                updated_at: meal.updated_at,
            };

            (StatusCode::OK, Json(json!(augmented_meal)))
        }
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

#[derive(TryFromMultipart)]
pub struct UpdateMealPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub is_available: Option<bool>,
    #[form_data(limit = "10MiB")]
    cover_image: Option<FieldData<NamedTempFile>>,
}

async fn update_meal_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    TypedMultipart(payload): TypedMultipart<UpdateMealPayload>,
) -> impl IntoResponse {
    let kitchen = match kitchen::repository::find_by_owner_id(
        &ctx.db_conn.pool,
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

    let meal = match repository::find_by_id(&ctx.db_conn.pool, id.clone()).await {
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

    if !repository::is_owner(&auth.user, &kitchen, &meal) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this meal"})),
        );
    }

    let cover_image =
        match payload.cover_image {
            Some(mut cover_image) => {
                let mut buf: Vec<u8> = vec![];

                if let Err(err) = cover_image.contents.read_to_end(&mut buf) {
                    tracing::error!("Failed to read the uploaded file {:?}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to upload image" })),
                    );
                }

                let media =
                    match storage::update_file(ctx.storage.clone(), meal.cover_image.clone(), buf)
                        .await
                    {
                        Ok(media) => media,
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "error": "Failed to upload image" })),
                            );
                        }
                    };

                Some(media)
            }
            None => None,
        };

    match repository::update_by_id(
        &ctx.db_conn.pool,
        id,
        repository::UpdateMealPayload {
            name: payload.name,
            description: payload.description,
            price: payload.price.map(|price| price.0),
            rating: None,
            is_available: payload.is_available,
            cover_image,
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
    match repository::find_by_id(&ctx.db_conn.pool, id.clone()).await {
        Ok(maybe_meal) => {
            if maybe_meal.is_none() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Meal not found"})),
                );
            }

            let meal = maybe_meal.unwrap();
            let kitchen = if let Ok(Some(kitchen)) =
                kitchen::repository::find_by_id(&ctx.db_conn.pool, meal.kitchen_id.clone()).await
            {
                kitchen
            } else {
                return (
                    StatusCode::OK,
                    Json(json!({ "error": "Kitchen not found" })),
                );
            };

            if !repository::is_owner(&auth.user, &kitchen, &meal) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "You are not the owner of this kitchen"})),
                );
            }

            if let Err(_) =
                storage::delete_file(ctx.storage.clone(), meal.cover_image.clone()).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete meal" })),
                );
            }

            match repository::delete_by_id(&ctx.db_conn.pool, id.clone()).await {
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
    match repository::like_by_id(&ctx.db_conn.pool, id, auth.user.id).await {
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
    match repository::unlike_by_id(&ctx.db_conn.pool, id, auth.user.id).await {
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
