use super::types::{request, response};
use crate::{
    modules::{
        kitchen, notification,
        order::repository::{self, Order, OrderStatus},
        transaction, user, wallet,
    },
    types::Context,
};
use bigdecimal::BigDecimal;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;

async fn mark_order_as(
    tx: &mut Transaction<'_, Postgres>,
    order: &Order,
    status: repository::OrderStatus,
) -> Result<(), response::Error> {
    repository::update_order_status(&mut **tx, order.id.clone(), status)
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or(response::Error::FailedToUpdateOrderStatus)
        .map(|_| ())
}

async fn process_order_delivery_finances(
    tx: &mut Transaction<'_, Postgres>,
    order: &Order,
) -> Result<(), response::Error> {
    let vendor_amount = order.total.clone() / (BigDecimal::from(12) / BigDecimal::from(10));

    let wallet = wallet::repository::find_by_kitchen_id(&mut **tx, order.kitchen_id.clone())
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or(response::Error::FailedToUpdateOrderStatus)?;

    let initial_order_payment_transaction =
        transaction::repository::find_initial_order_payment_transaction_by_order_id(
            &mut **tx,
            order.id.clone(),
        )
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or_else(|| {
            tracing::error!("Required a transaction for an order which doesn't have an initial payment transaction: {}", &order.id);
            response::Error::FailedToUpdateOrderStatus
        })?;

    transaction::repository::create(
        &mut **tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: vendor_amount.clone(),
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some(format!("Payment received for order {}", order.id.clone())),
                purpose: Some(transaction::repository::TransactionPurpose::Order(
                    transaction::repository::TransactionPurposeOrder {
                        order_id: order.id.clone(),
                    },
                )),
                r#ref: Some(initial_order_payment_transaction.r#ref),
                wallet_id: wallet.id.clone(),
                user_id: order.owner_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateOrderStatus)?;

    wallet::repository::update_by_id(
        &mut **tx,
        wallet.id,
        wallet::repository::UpdateByIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: vendor_amount,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateOrderStatus)
}

async fn process_order_cancellation_finances(
    tx: &mut Transaction<'_, Postgres>,
    order: &Order,
) -> Result<(), response::Error> {
    let wallet = wallet::repository::find_by_owner_id(&mut **tx, order.owner_id.clone())
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or(response::Error::FailedToUpdateOrderStatus)?;

    let initial_order_payment_transaction =
        transaction::repository::find_initial_order_payment_transaction_by_order_id(
            &mut **tx,
            order.id.clone(),
        )
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or_else(||{
            tracing::error!("Required a transaction for an order which doesn't have an initial payment transaction: {}", &order.id);
            response::Error::FailedToUpdateOrderStatus
        })?;

    transaction::repository::create(
        &mut **tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: initial_order_payment_transaction.amount.clone(),
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some(format!(
                    "Payment refunded for order {} cancellation",
                    order.id.clone()
                )),
                purpose: Some(transaction::repository::TransactionPurpose::Order(
                    transaction::repository::TransactionPurposeOrder {
                        order_id: order.id.clone(),
                    },
                )),
                r#ref: Some(initial_order_payment_transaction.r#ref),
                wallet_id: wallet.id.clone(),
                user_id: order.owner_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateOrderStatus)?;

    wallet::repository::update_by_id(
        &mut **tx,
        wallet.id,
        wallet::repository::UpdateByIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: order.total.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToUpdateOrderStatus)
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let order = repository::find_by_id(&ctx.db_conn.pool, payload.id.clone())
        .await
        .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
        .ok_or(response::Error::OrderNotFound)?;

    let mut tx = ctx.db_conn.clone().pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {}", err);
        response::Error::FailedToUpdateOrderStatus
    })?;

    let as_kitchen = payload.body.as_kitchen.unwrap_or(false);

    if as_kitchen {
        let kitchen =
            kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id.clone())
                .await
                .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
                .ok_or(response::Error::UserNotOwnKitchen)?;

        if kitchen.id != order.kitchen_id {
            return Err(response::Error::KitchenNotOwner);
        }

        match (order.status.clone(), payload.body.status.clone()) {
            (
                repository::OrderStatus::AwaitingAcknowledgement,
                repository::OrderStatus::Preparing,
            )
            | (
                repository::OrderStatus::AwaitingAcknowledgement,
                repository::OrderStatus::Cancelled,
            )
            | (repository::OrderStatus::Preparing, repository::OrderStatus::InTransit) => {
                match payload.body.status.clone() {
                    repository::OrderStatus::Cancelled => {
                        process_order_cancellation_finances(&mut tx, &order).await?;
                    }
                    _ => (),
                };

                let order = repository::update_order_status(
                    &mut *tx,
                    order.id.clone(),
                    payload.body.status.clone(),
                )
                .await
                .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
                .ok_or(response::Error::OrderNotFound)?;

                let order_owner = user::repository::find_by_id(&mut *tx, order.owner_id.clone())
                    .await
                    .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
                    .ok_or(response::Error::FailedToUpdateOrderStatus)?;

                notification::service::send(
                    ctx,
                    notification::service::Notification::order_status_updated(order, order_owner),
                    notification::service::Backend::Push,
                )
                .await;
            }
            _ => {
                return Err(response::Error::InvalidStatusTransitionForKitchen);
            }
        }
    } else {
        if !repository::is_owner(&order, &payload.auth.user) {
            return Err(response::Error::UserNotOwner);
        }

        let kitchen_owner =
            user::repository::find_by_kitchen_id(&mut *tx, order.kitchen_id.clone())
                .await
                .map_err(|_| response::Error::FailedToUpdateOrderStatus)?
                .ok_or(response::Error::FailedToUpdateOrderStatus)?;

        match (order.status.clone(), payload.body.status.clone()) {
            (repository::OrderStatus::InTransit, repository::OrderStatus::Delivered) => {
                process_order_delivery_finances(&mut tx, &order).await;
                mark_order_as(&mut tx, &order, repository::OrderStatus::Delivered).await?;

                notification::service::send(
                    ctx,
                    notification::service::Notification::order_status_updated(order, kitchen_owner),
                    notification::service::Backend::Push,
                )
                .await;
            }
            (
                repository::OrderStatus::AwaitingAcknowledgement,
                repository::OrderStatus::Cancelled,
            ) => {
                process_order_cancellation_finances(&mut tx, &order).await?;
                mark_order_as(&mut tx, &order, repository::OrderStatus::Cancelled).await?;

                notification::service::send(
                    ctx,
                    notification::service::Notification::order_status_updated(order, kitchen_owner),
                    notification::service::Backend::Push,
                )
                .await;
            }
            _ => return Err(response::Error::InvalidStatusTransitionForUser),
        }
    };

    tx.commit()
        .await
        .map_err(|err| {
            tracing::error!("Failed to commit database transaction: {}", err);
            response::Error::FailedToUpdateOrderStatus
        })
        .map(|_| response::Success::OrderStatusUpdated)
}
