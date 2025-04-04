use super::types::{request, response};
use crate::{
    modules::{
        auth::middleware::Auth,
        cart::repository::{self, CartItem, CartItems, UpdateCartPayload},
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
        repository::find_active_full_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
            .await
            .map_err(|_| response::Error::FailedToFindCart)?
            .ok_or(response::Error::CartNotFound)?;

    let filtered_cart_items = cart
        .items
        .0
        .into_iter()
        .filter(|item| item.kitchen.id != payload.kitchen_id)
        .map(|item| CartItem {
            meal_id: item.meal.id,
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();

    repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id,
        UpdateCartPayload {
            items: Some(CartItems(filtered_cart_items)),
            status: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToRemoveItemsFromCart)
    .map(|_| response::Success::ItemsRemovedFromCart)
}
