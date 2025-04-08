use super::types::{request, response};
use crate::{
    modules::{
        auth::middleware::Auth,
        cart::repository::{self, CartItems},
        meal,
    },
    types::Context,
};
use std::sync::Arc;

pub async fn service(
    ctx: Arc<Context>,
    auth: Auth,
    payload: request::Payload,
) -> response::Response {
    let cart = repository::find_active_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToFindCart)?
        .ok_or(response::Error::CartNotFound)?;

    let meal = meal::repository::find_by_id(&ctx.db_conn.pool, payload.meal_id)
        .await
        .map_err(|_| response::Error::FailedToFetchMeal)?
        .ok_or(response::Error::MealNotFound)?;

    let mut found = false;

    let new_items = cart
        .items
        .0
        .into_iter()
        .filter(|item| {
            if item.meal_id == meal.id {
                found = true;
                return false;
            }

            true
        })
        .collect::<Vec<_>>();

    if !found {
        return Err(response::Error::MealNotFoundInCart);
    }

    repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id.clone(),
        repository::UpdateCartPayload {
            items: Some(CartItems(new_items)),
            status: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToRemoveMealFromCart)
    .map(|_| response::Success::MealRemovedFromCart)
}
