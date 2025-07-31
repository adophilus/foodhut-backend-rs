pub mod online;

use crate::modules::order::repository::{Order, OrderStatus};
use crate::modules::{kitchen, notification, order, transaction, user, wallet};
use crate::{modules::user::repository::User, types::Context};
use serde::Serialize;
use serde_json::json;
use sqlx::{PgExecutor, Postgres, Transaction};
use std::sync::Arc;

use std::borrow::BorrowMut;

pub enum Error {
    UnexpectedError,
    AlreadyPaid,
    InsufficientBalance,
}

pub enum PaymentMethod {
    Wallet,
    Online,
}

impl From<order::repository::PaymentMethod> for PaymentMethod {
    fn from(method: order::repository::PaymentMethod) -> Self {
        match method {
            order::repository::PaymentMethod::Online => PaymentMethod::Online,
            order::repository::PaymentMethod::Wallet => PaymentMethod::Wallet,
        }
    }
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
    if payload.order.status != OrderStatus::AwaitingPayment {
        return Err(Error::AlreadyPaid);
    }

    match payload.method {
        PaymentMethod::Wallet => wallet::service::initialize_payment_for_order(
            ctx,
            &mut tx,
            wallet::service::InitializePaymentForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        .map(|_| PaymentDetails(json!({ "message": "Payment successful" })))
        .map_err(|err| match err {
            wallet::service::Error::InsufficientBalance => Error::InsufficientBalance,
            _ => Error::UnexpectedError,
        }),
        PaymentMethod::Online => online::initialize_invoice_for_order(
            ctx,
            online::InitializeInvoiceForOrder {
                order: payload.order,
                payer: payload.payer,
            },
        )
        .await
        .map(|details| PaymentDetails(json!(details)))
        .map_err(|_| Error::UnexpectedError),
    }
}

pub struct ConfirmPaymentForOrderPayload {
    pub order: Order,
    pub payment_method: PaymentMethod,
}

pub async fn confirm_payment_for_order(
    ctx: Arc<Context>,
    tx: &mut Transaction<'_, Postgres>,
    payload: ConfirmPaymentForOrderPayload,
) -> Result<(), Error> {
    match payload.payment_method {
        PaymentMethod::Online => online::confirm_payment_for_order(
            tx,
            online::ConfirmPaymentForOrderPayload {
                order: payload.order.clone(),
            },
        )
        .await
        .map_err(|_| Error::UnexpectedError)?,
        PaymentMethod::Wallet => {
            let payer_wallet =
                wallet::repository::find_by_owner_id(&mut **tx, payload.order.owner_id.clone())
                    .await
                    .map_err(|_| Error::UnexpectedError)?
                    .ok_or(Error::UnexpectedError)?;
            wallet::service::confirm_payment_for_order(
                tx,
                wallet::service::ConfirmPaymentForOrderPayload {
                    order: payload.order.clone(),
                    wallet: payer_wallet,
                },
            )
            .await
            .map_err(|err| match err {
                wallet::service::Error::InsufficientBalance => Error::InsufficientBalance,
                _ => Error::UnexpectedError,
            })?;
        }
    }

    tracing::info!(
        "Transaction successful for order {}",
        payload.order.id.clone()
    );

    order::repository::confirm_payment(
        &mut **tx,
        order::repository::ConfirmPaymentPayload {
            order_id: payload.order.id.clone(),
            payment_method: payload.payment_method.into(),
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    let kitchen = kitchen::repository::find_by_id(&mut **tx, payload.order.kitchen_id.clone())
        .await
        .map_err(|_| Error::UnexpectedError)?
        .ok_or(Error::UnexpectedError)?;

    let kitchen_owner = user::repository::find_by_id(&mut **tx, kitchen.owner_id)
        .await
        .map_err(|_| Error::UnexpectedError)?
        .ok_or(Error::UnexpectedError)?;

    tokio::spawn(notification::service::send(
        ctx.clone(),
        notification::service::Notification::order_status_updated(
            payload.order.clone(),
            kitchen_owner,
        ),
        notification::service::Backend::Push,
    ));

    let admin_users = user::repository::find_all_admins(&mut **tx)
        .await
        .map_err(|_| Error::UnexpectedError)?;

    for admin_user in admin_users {
        tokio::spawn(notification::service::send(
            ctx.clone(),
            notification::service::Notification::order_status_updated(
                payload.order.clone(),
                admin_user,
            ),
            notification::service::Backend::Push,
        ));
    }

    Ok(())
}
