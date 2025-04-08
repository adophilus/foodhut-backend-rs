use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, cart, meal::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    match payload.auth {
        Some(auth) => {
            let cart = match cart::repository::find_active_cart_by_owner_id(
                &ctx.db_conn.pool,
                auth.user.id.clone(),
            )
            .await
            .map_err(|_| response::Error::FailedToFetchMeals)?
            {
                Some(cart) => cart,
                _ => match cart::repository::create(
                    &ctx.db_conn.pool,
                    cart::repository::CreateCartPayload {
                        owner_id: auth.user.id.clone(),
                    },
                )
                .await
                {
                    Ok(cart) => cart,
                    Err(_) => return Err(response::Error::FailedToFetchMeals),
                },
            };

            let is_liked_by = match payload.filters.is_liked {
                Some(true) => Some(auth.user.id.clone()),
                _ => None,
            };

            let paginated_meals = if user::repository::is_admin(&auth.user) {
                repository::find_many_as_admin(
                    &ctx.db_conn.pool,
                    payload.pagination,
                    repository::FindManyAsAdminFilters {
                        kitchen_id: payload.filters.kitchen_id,
                        search: payload.filters.search,
                        is_liked_by,
                    },
                )
                .await
            } else if payload.filters.as_kitchen.unwrap_or(false) {
                repository::find_many_as_kitchen(
                    &ctx.db_conn.pool,
                    payload.pagination,
                    repository::FindManyAsKitchenFilters {
                        kitchen_id: payload.filters.kitchen_id,
                        search: payload.filters.search,
                        is_liked_by,
                        owner_id: auth.user.id.clone(),
                    },
                )
                .await
            } else {
                repository::find_many_as_user(
                    &ctx.db_conn.pool,
                    payload.pagination,
                    repository::FindManyAsUserFilters {
                        kitchen_id: payload.filters.kitchen_id,
                        search: payload.filters.search,
                        is_liked_by,
                    },
                )
                .await
            };

            match paginated_meals {
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
                            deleted_at: meal.deleted_at,
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
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch meals"})),
                ),
            }
        }
        None => {
            let paginated_meals = repository::find_many_as_user(
                &ctx.db_conn.pool,
                payload.pagination,
                repository::FindManyAsUserFilters {
                    kitchen_id: payload.filters.kitchen_id,
                    search: payload.filters.search,
                    is_liked_by: None,
                },
            )
            .await;

            match paginated_meals {
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
                            in_cart: false,
                            kitchen_id: meal.kitchen_id,
                            created_at: meal.created_at,
                            updated_at: meal.updated_at,
                            deleted_at: meal.deleted_at,
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
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch meals"})),
                ),
            }
        }
    }
}
