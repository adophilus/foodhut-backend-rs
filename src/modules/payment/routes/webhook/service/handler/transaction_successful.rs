use super::super::super::types::{
    response, Metadata, OrderInvoiceMetadata, TopupMetadata, TransactionSuccessful,
};
use crate::modules::{payment::service, wallet};
use crate::{
    modules::{order, transaction},
    types::Context,
};
use bigdecimal::BigDecimal;
use std::sync::Arc;

async fn successful_order_payment(
    ctx: Arc<Context>,
    amount: BigDecimal,
    metadata: OrderInvoiceMetadata,
) -> response::Response {
    let mut tx = ctx.db_conn.pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {:?}", err);
        response::Error::ServerError
    })?;

    let order = order::repository::find_by_id(&mut *tx, metadata.order_id.clone())
        .await
        .map_err(|_| response::Error::ServerError)?
        .ok_or_else(|| {
            tracing::error!(
                "Order not found for successful transactino: {}",
                &metadata.order_id
            );
            response::Error::OrderNotFound
        })?;

    if order.status != order::repository::OrderStatus::AwaitingPayment {
        return Ok(response::Success::Successful);
    }

    if amount / BigDecimal::from(100) < order.total {
        tracing::error!(
            "Payload order amount is less than order total: {}",
            &order.id
        );
        return Err(response::Error::InvalidPayload);
    }

    // let cart = match repository::cart::find_by_id(&mut *tx, order.cart_id.clone()).await {
    //     Ok(Some(cart)) => cart,
    //     Ok(None) => return StatusCode::NOT_FOUND.into_response(),
    //     Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    // };

    service::confirm_payment_for_order(
        ctx.clone(),
        &mut tx,
        service::ConfirmPaymentForOrderPayload {
            order: order.clone(),
            payment_method: service::PaymentMethod::Online,
        },
    )
    .await
    .map_err(|_| response::Error::ServerError)?;

    tracing::info!("Transaction successful for order {}", order.id.clone());

    tx.commit()
        .await
        .map(|_| response::Success::Successful)
        .map_err(|err| {
            tracing::error!("Failed to commit database transaction: {:?}", err);
            response::Error::ServerError
        })
}

async fn successful_topup(
    ctx: Arc<Context>,
    amount: BigDecimal,
    metadata: TopupMetadata,
) -> response::Response {
    let mut tx = ctx.db_conn.pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start database transaction: {:?}", err);
        response::Error::ServerError
    })?;

    let topup_amount = amount / BigDecimal::from(100);

    wallet::repository::update_by_owner_id(
        &mut *tx,
        metadata.user_id.clone(),
        wallet::repository::UpdateByOwnerIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: topup_amount.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::ServerError);

    transaction::repository::create(
        &mut *tx,
        transaction::repository::CreatePayload::Online(
            transaction::repository::CreateOnlineTransactionPayload {
                amount: topup_amount,
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some("Topup".to_string()),
                purpose: Some(transaction::repository::TransactionPurpose::Other(
                    transaction::repository::TransactionPurposeOther,
                )),
                r#ref: None,
                user_id: metadata.user_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| response::Error::ServerError)?;

    tracing::debug!(
        "Topup Transaction successful for {}",
        metadata.user_id.clone()
    );

    tx.commit()
        .await
        .map(|_| response::Success::Successful)
        .map_err(|err| {
            tracing::error!("Failed to commit database transaction: {:?}", err);
            response::Error::ServerError
        })
}

pub async fn handler(ctx: Arc<Context>, event: TransactionSuccessful) -> response::Response {
    match event.metadata {
        Metadata::Order(metadata) => {
            successful_order_payment(ctx.clone(), event.amount, metadata).await
        }
        Metadata::Topup(metadata) => successful_topup(ctx.clone(), event.amount, metadata).await,
    }
}
