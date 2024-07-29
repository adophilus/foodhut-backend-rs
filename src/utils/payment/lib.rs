use crate::repository::{order::Order, user::User};
use crate::types::Context;
use crate::utils::{online, wallet};
use serde::Serialize;
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
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
pub struct PaymentDetails {
    url: Option<String>,
}

pub async fn initialize_payment_for_order(
    ctx: Arc<Context>,
    payload: InitializePaymentForOrder,
) -> Result<PaymentDetails, Error> {
    match payload.method {
        PaymentMethod::Wallet => match wallet::initialize_payment_for_order(
            ctx.db_conn.clone(),
            wallet::InitializePaymentForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        {
            Ok(_) => Ok(PaymentDetails { url: None }),
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
            // FIX: this is meant to return the correct url
            Ok(_) => Ok(PaymentDetails { url: None }),
            Err(_) => Err(Error::UnexpectedError),
        },
    }
}
