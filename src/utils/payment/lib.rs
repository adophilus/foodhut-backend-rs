use crate::repository;
use crate::repository::{order::Order, user::User};
use crate::types::Context;
use crate::utils::{online, wallet};
use serde::Serialize;
use serde_json::json;
use sqlx::{PgExecutor, Postgres, Transaction};
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
    AlreadyPaid,
}

pub enum PaymentMethod {
    Wallet,
    Online,
}

pub struct InitializePaymentForOrder {
    pub method: PaymentMethod,
    pub payer: User,
    pub order: Order,
}

#[derive(Serialize)]
pub struct PaymentDetails(serde_json::Value);

pub async fn initialize_payment_for_order(
    ctx: Arc<Context>,
    mut tx: &mut Transaction<'_, Postgres>,
    payload: InitializePaymentForOrder,
) -> Result<PaymentDetails, Error> {
    if payload.order.status != repository::order::OrderStatus::AwaitingPayment {
        return Err(Error::AlreadyPaid);
    }

    match payload.method {
        PaymentMethod::Wallet => match wallet::initialize_payment_for_order(
            &mut tx,
            wallet::InitializePaymentForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        {
            Ok(_) => Ok(PaymentDetails(json!({ "message": "Payment successful" }))),
            Err(_) => Err(Error::UnexpectedError),
        },
        PaymentMethod::Online => match online::initialize_payment_for_order(
            ctx,
            online::InitializePaymentForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        {
            Ok(details) => Ok(PaymentDetails(json!(details))),
            Err(_) => Err(Error::UnexpectedError),
        },
    }
}

pub async fn confirm_payment_for_order<'e, E: PgExecutor<'e>>(
    ctx: Arc<Context>,
    e: E,
    order: repository::order::Order,
) -> Result<(), Error> {
    if let Err(_) = repository::order::confirm_payment(e, order.id.clone()).await
    {
        return Err(Error::UnexpectedError);
    };

    // TODO: send notification to the end user

    Ok(())
}
