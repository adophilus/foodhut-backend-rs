use super::types::{request, response};
use crate::{
    modules::{
        cart,
        meal::repository::{self, MealWithCartStatus},
        user,
    },
    types::Context,
    utils::pagination::Paginated,
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

            paginated_meals
                .map_err(|_| response::Error::FailedToFetchMeals)
                .map(|meals| {
                    let augmented_meals = meals
                        .items
                        .clone()
                        .into_iter()
                        .map(|meal| meal.with_cart_status(&cart))
                        .collect::<Vec<_>>();

                    response::Success::Meals(Paginated::new(
                        augmented_meals,
                        meals.meta.total,
                        meals.meta.page,
                        meals.meta.per_page,
                    ))
                })
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

            paginated_meals
                .map_err(|_| response::Error::FailedToFetchMeals)
                .map(|meals| {
                    let augmented_meals = meals
                        .items
                        .clone()
                        .into_iter()
                        .map(|meal| meal.into())
                        .collect::<Vec<MealWithCartStatus>>();

                    response::Success::Meals(Paginated::new(
                        augmented_meals,
                        meals.meta.total,
                        meals.meta.page,
                        meals.meta.per_page,
                    ))
                })
        }
    }
}
