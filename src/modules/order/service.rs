use crate::modules::payment::service::PaymentDetails;
use crate::modules::user::repository::User;
use crate::modules::{payment, transaction, wallet};
use crate::types::Context;
use bigdecimal::BigDecimal;
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
    payload: MarkOrderAsDeliveredPayload,
) -> Result<(), Error> {
    let mut tx = match ctx.db_conn.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err(Error::UnexpectedError),
    };

    let vendor_amount = payload.order.total / (BigDecimal::from(12) / BigDecimal::from(10));

    match repository::update_order_status(
        &mut *tx,
        payload.order.id.clone(),
        OrderStatus::Delivered,
    )
    .await
    {
        Ok(true) => (),
        Ok(false) => Err(Error::UnexpectedError)?,
        _ => Err(Error::UnexpectedError)?,
    };

    let wallet =
        match wallet::repository::find_by_kitchen_id(&mut *tx, payload.order.kitchen_id.clone())
            .await
        {
            Ok(Some(wallet)) => wallet,
            _ => Err(Error::UnexpectedError)?,
        };

    transaction::repository::create(
        &mut *tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: vendor_amount.clone(),
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some(format!(
                    "Payment received for order {}",
                    payload.order.id.clone()
                )),
                wallet_id: wallet.id.clone(),
                user_id: payload.order.owner_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    wallet::repository::update_by_id(
        &mut *tx,
        wallet.id,
        wallet::repository::UpdateByIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: vendor_amount,
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    tx.commit().await.map_err(|_| Error::UnexpectedError)?;

    Ok(())
}
