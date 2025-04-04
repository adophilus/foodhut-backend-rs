use super::types::{
    request,
    response::{self, KitchenWithMeals, MealWithQuantity},
};
use crate::{
    modules::{
        auth::middleware::Auth, cart::repository, kitchen::repository::Kitchen,
        meal::repository::MealWithCartStatus,
    },
    types::Context,
};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc};

pub async fn service(ctx: Arc<Context>, auth: Auth) -> response::Response {
    repository::find_active_full_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToFetchActiveCart)?
        .ok_or(response::Error::CartNotFound)
        .map(|cart| {
            let kitchen_id_to_kitchen_map = cart
                .items
                .0
                .clone()
                .into_iter()
                .map(|item| item.kitchen.clone())
                .unique_by(|kitchen| kitchen.id.clone())
                .map(|kitchen| (kitchen.id.clone(), kitchen))
                .collect::<HashMap<String, Kitchen>>();

            let items = kitchen_id_to_kitchen_map
                .into_iter()
                .map(|(id, kitchen)| {
                    let meals = cart
                        .items
                        .0
                        .clone()
                        .into_iter()
                        .filter(|item| item.kitchen.id == id)
                        .map(|item| MealWithQuantity {
                            quantity: item.quantity,
                            meal: MealWithCartStatus {
                                id: item.meal.id,
                                name: item.meal.name,
                                description: item.meal.description,
                                rating: item.meal.rating,
                                original_price: item.meal.original_price,
                                price: item.meal.price,
                                likes: item.meal.likes,
                                cover_image: item.meal.cover_image,
                                is_available: item.meal.is_available,
                                kitchen_id: item.meal.kitchen_id,
                                created_at: item.meal.created_at,
                                updated_at: item.meal.updated_at,
                                deleted_at: item.meal.deleted_at,
                                in_cart: true,
                            },
                        })
                        .collect::<Vec<_>>();

                    KitchenWithMeals { kitchen, meals }
                })
                .collect::<Vec<_>>();

            response::Success::Items(items)
        })
}
