use crate::modules::payment::service::PaymentDetails;
use crate::modules::user::repository::User;
use crate::modules::{payment, transaction, wallet};
use crate::types::Context;
use bigdecimal::BigDecimal;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;

use super::repository::{self, Order, OrderStatus};

pub enum Error {
    UnexpectedError,
}

pub struct PayForOrderPayload {
    pub payment_method: repository::PaymentMethod,
    pub order: Order,
    pub payer: User,
}

pub enum PayForOrderError {
    UnexpectedError,
    AlreadyPaid,
}

pub async fn pay_for_order(
    ctx: Arc<Context>,
    payload: PayForOrderPayload,
) -> Result<PaymentDetails, PayForOrderError> {
    let method = match payload.payment_method {
        repository::PaymentMethod::Online => payment::service::PaymentMethod::Online,
        repository::PaymentMethod::Wallet => payment::service::PaymentMethod::Wallet,
    };

    let mut tx = ctx.db_conn.pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start transaction: {}", err);
        PayForOrderError::UnexpectedError
    })?;

    let details = payment::service::initialize_payment_for_order(
        ctx.clone(),
        &mut tx,
        payment::service::InitializePaymentForOrder {
            method,
            order: payload.order,
            payer: payload.payer.clone(),
        },
    )
    .await
    .map_err(|err| match err {
        payment::service::Error::AlreadyPaid => PayForOrderError::AlreadyPaid,
        _ => PayForOrderError::UnexpectedError,
    })?;

    tx.commit().await.map_err(|err| {
        tracing::error!("Failed to commit transaction: {}", err);
        PayForOrderError::UnexpectedError
    })?;

    Ok(details)
}

pub struct MarkOrderAsDeliveredPayload {
    pub order: Order,
}

pub async fn mark_order_as_delivered(
    ctx: Arc<Context>,
    tx: &mut Transaction<'_, Postgres>,
    payload: MarkOrderAsDeliveredPayload,
) -> Result<(), Error> {
    let vendor_amount = payload.order.total / (BigDecimal::from(12) / BigDecimal::from(10));

    match repository::update_order_status(
        &mut **tx,
        payload.order.id.clone(),
        OrderStatus::Delivered,
    )
    .await
    {
        Ok(Some(_)) => (),
        _ => Err(Error::UnexpectedError)?,
    };

    let wallet =
        match wallet::repository::find_by_kitchen_id(&mut **tx, payload.order.kitchen_id.clone())
            .await
        {
            Ok(Some(wallet)) => wallet,
            _ => Err(Error::UnexpectedError)?,
        };

    let initial_order_payment_transaction =
        transaction::repository::find_initial_order_payment_transaction_by_order_id(
            &mut **tx,
            payload.order.id.clone(),
        )
        .await
        .map_err(|_| Error::UnexpectedError)?;

    if initial_order_payment_transaction.is_none() {
        tracing::error!("Required a transaction for an order which doesn't have an initial payment transaction: {}", &payload.order.id);
        return Err(Error::UnexpectedError);
    }

    let initial_order_payment_transaction = initial_order_payment_transaction.unwrap();

    transaction::repository::create(
        &mut **tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: vendor_amount.clone(),
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some(format!(
                    "Payment received for order {}",
                    payload.order.id.clone()
                )),
                purpose: Some(transaction::repository::TransactionPurpose::Order(
                    transaction::repository::TransactionPurposeOrder {
                        order_id: payload.order.id,
                    },
                )),
                r#ref: Some(initial_order_payment_transaction.r#ref),
                wallet_id: wallet.id.clone(),
                user_id: payload.order.owner_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    wallet::repository::update_by_id(
        &mut **tx,
        wallet.id,
        wallet::repository::UpdateByIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: vendor_amount,
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    Ok(())
}
