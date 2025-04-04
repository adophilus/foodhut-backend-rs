use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, cart::repository, order},
    types::Context,
};
use std::sync::Arc;

pub async fn service(
    ctx: Arc<Context>,
    auth: Auth,
    payload: request::Payload,
) -> response::Response {
    let parsed_delivery_date = payload
        .body
        .delivery_date
        .map(|d| chrono::NaiveDateTime::parse_from_str(&d.to_string(), "%s"))
        .transpose()
        .map_err(|err| response::Error::InvalidDate(err.to_string()))?;

    parsed_delivery_date
        .clone()
        .map(|delivery_date| {
            if delivery_date < chrono::Utc::now().naive_utc() {
                Err(response::Error::InvalidDate(String::from(
                    "Delivery date cannot be in the past",
                )))
            } else {
                Ok(())
            }
        })
        .transpose()?;

    let cart =
        repository::find_active_full_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
            .await
            .map_err(|_| response::Error::FailedToFindCart)?
            .ok_or(response::Error::CartNotFound)?;

    let items_to_checkout = cart
        .items
        .0
        .into_iter()
        .filter(|item| item.kitchen.id == payload.kitchen_id)
        .collect::<Vec<_>>();

    if items_to_checkout.len() == 0 {
        return Err(response::Error::NoItemsToCheckout);
    }

    let mut tx = ctx.db_conn.clone().pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {}", err);
        response::Error::FailedToCheckoutCart
    })?;

    let order = order::repository::create(
        &mut *tx,
        order::repository::CreateOrderPayload {
            items: items_to_checkout,
            payment_method: payload.body.payment_method.clone(),
            delivery_address: payload.body.delivery_address.clone(),
            delivery_date: parsed_delivery_date.clone(),
            dispatch_rider_note: payload.body.dispatch_rider_note.clone(),
            kitchen_id: payload.kitchen_id,
            owner_id: auth.user.id.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCheckoutCart)?;

    repository::update_by_id(
        &mut *tx,
        cart.id.clone(),
        repository::UpdateCartPayload {
            items: None,
            status: Some(repository::CartStatus::CheckedOut),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCheckoutCart)?;

    tx.commit()
        .await
        .map_err(|err| {
            tracing::error!("Failed to commit transaction: {}", err);
            response::Error::FailedToCheckoutCart
        })
        .map(|_| response::Success::CheckoutSuccessful(order))
}
