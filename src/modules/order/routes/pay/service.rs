use super::types::{request, response};
use crate::{
    modules::{
        order::repository::{self, Order},
        payment,
        user::repository::User,
    },
    types::Context,
};
use std::sync::Arc;

pub struct PayForOrderPayload {
    pub payment_method: repository::PaymentMethod,
    pub order: Order,
    pub payer: User,
}

pub async fn pay_for_order(ctx: Arc<Context>, payload: PayForOrderPayload) -> response::Response {
    let method = match payload.payment_method {
        repository::PaymentMethod::Online => payment::service::PaymentMethod::Online,
        repository::PaymentMethod::Wallet => payment::service::PaymentMethod::Wallet,
    };

    let mut tx = ctx.db_conn.pool.begin().await.map_err(|err| {
        tracing::error!("Failed to start transaction: {}", err);
        response::Error::FailedToInitiateOrderPayment
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
        payment::service::Error::AlreadyPaid => response::Error::PaymentAlreadyMade,
        _ => response::Error::FailedToInitiateOrderPayment,
    })?;

    tx.commit()
        .await
        .map_err(|err| {
            tracing::error!("Failed to commit transaction: {}", err);
            response::Error::FailedToInitiateOrderPayment
        })
        .map(|_| response::Success::PaymentDetails(details))
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let order = repository::find_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToInitiateOrderPayment)?
        .ok_or(response::Error::OrderNotFound)?;

    pay_for_order(
        ctx.clone(),
        PayForOrderPayload {
            payment_method: payload.body.with,
            order,
            payer: payload.auth.user,
        },
    )
    .await
}
