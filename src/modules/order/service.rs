use crate::modules::{transaction, wallet};
use crate::types::Context;
use bigdecimal::BigDecimal;
use std::sync::Arc;

use super::repository::{self, Order, OrderStatus};

pub enum Error {
    UnexpectedError,
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

    let wallet = match wallet::repository::find_by_owner_id(
        &mut *tx,
        payload.order.kitchen_id.clone(),
    )
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
