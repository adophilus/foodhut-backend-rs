use super::types::{request, response};
use crate::{
    modules::{
        auth::middleware::Auth,
        cart::repository::{self, CartItem, CartItems},
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
    let cart =
        match repository::find_active_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
            .await
            .map_err(|_| response::Error::FailedToSetItemInCart)?
        {
            Some(cart) => cart,
            _ => repository::create(
                &ctx.db_conn.pool,
                repository::CreateCartPayload {
                    owner_id: auth.user.id.clone(),
                },
            )
            .await
            .map_err(|_| response::Error::FailedToSetItemInCart)?,
        };

    let meal = meal::repository::find_by_id(&ctx.db_conn.pool, payload.meal_id)
        .await
        .map_err(|_| response::Error::FailedToFetchMeal)?
        .ok_or(response::Error::MealNotFound)?;

    let mut found = false;
    let mut index = 0;

    let mut items = cart.items.0;
    for item in items.iter_mut() {
        if item.meal_id == meal.id.clone() {
            found = true;
            item.quantity = payload.body.quantity;
            break;
        }
        index += 1;
    }

    if !found {
        items.push(CartItem {
            meal_id: meal.id.clone(),
            quantity: payload.body.quantity,
        });
    } else {
        if payload.body.quantity == 0 {
            items.remove(index);
        }
    }

    repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id.clone(),
        repository::UpdateCartPayload {
            items: Some(CartItems(items)),
            status: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateCart)
    .map(|_| response::Success::CartUpdated)
}
