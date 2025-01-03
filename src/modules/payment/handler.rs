use super::{model, service};
use crate::modules::{notification, order, transaction, user, wallet};
use axum::response::IntoResponse;
use bigdecimal::{BigDecimal, FromPrimitive};
use hyper::StatusCode;
use std::sync::Arc;

use crate::types;

pub async fn successful_order_payment(
    ctx: Arc<types::Context>,
    amount: BigDecimal,
    metadata: service::online::OrderInvoiceMetadata,
) -> impl IntoResponse {
    let mut tx = match ctx.db_conn.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let order = match order::repository::find_by_id(&mut *tx, metadata.order_id).await {
        Ok(Some(order)) => order,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if order.status != order::repository::OrderStatus::AwaitingPayment {
        return StatusCode::OK.into_response();
    }

    if amount / BigDecimal::from(100) < order.total {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // let cart = match repository::cart::find_by_id(&mut *tx, order.cart_id.clone()).await {
    //     Ok(Some(cart)) => cart,
    //     Ok(None) => return StatusCode::NOT_FOUND.into_response(),
    //     Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    // };

    if let Err(_) = service::confirm_payment_for_order(
        ctx.clone(),
        &mut tx,
        service::ConfirmPaymentForOrderPayload {
            order: order.clone(),
            payment_method: service::PaymentMethod::Online,
        },
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    tracing::info!("Transaction successful for order {}", order.id.clone());

    match tx.commit().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            tracing::error!("Failed to commit transaction: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn successful_topup(
    ctx: Arc<types::Context>,
    amount: BigDecimal,
    metadata: service::online::TopupMetadata,
) -> impl IntoResponse {
    let mut tx = match ctx.db_conn.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let topup_amount = amount / BigDecimal::from(100);

    if let Err(_) = wallet::repository::update_by_owner_id(
        &mut *tx,
        metadata.user_id.clone(),
        wallet::repository::UpdateByOwnerIdPayload {
            operation: wallet::repository::UpdateOperation::Credit,
            amount: topup_amount.clone(),
        },
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    if let Err(_) = transaction::repository::create(
        &mut *tx,
        transaction::repository::CreatePayload::Online(
            transaction::repository::CreateOnlineTransactionPayload {
                amount: topup_amount,
                direction: transaction::repository::TransactionDirection::Incoming,
                note: Some("Topup".to_string()),
                user_id: metadata.user_id.clone(),
            },
        ),
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    tracing::debug!(
        "Topup Transaction successful for {}",
        metadata.user_id.clone()
    );

    match tx.commit().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            tracing::error!("Failed to commit transaction: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// pub async fn customer_identification_successful(
//     ctx: Arc<types::Context>,
//     payload: model::CustomerIdentificationSuccessful,
// ) -> impl IntoResponse {
//     let user = match repository::user::find_by_email(ctx.db_conn.clone(), payload.email).await {
//         Ok(Some(user)) => user,
//         _ => return StatusCode::INTERNAL_SERVER_ERROR,
//     };
//
//     match utils::wallet::service::request_bank_account_creation(ctx.clone(), user.clone()).await {
//         Ok(_) => StatusCode::OK,
//         _ => StatusCode::INTERNAL_SERVER_ERROR,
//     }
// }
//
// pub async fn customer_identification_failed(
//     ctx: Arc<types::Context>,
//     payload: model::CustomerIdentificationFailed,
// ) -> impl IntoResponse {
//     let user = match repository::user::find_by_email(ctx.db_conn.clone(), payload.email.clone())
//         .await
//     {
//         Ok(Some(user)) => user,
//         Ok(None) => {
//             return StatusCode::NOT_FOUND;
//         }
//         Err(_) => {
//             return StatusCode::INTERNAL_SERVER_ERROR;
//         }
//     };
//
//     let _ = notification::service::send(
//         ctx.clone(),
//         notification::service::Notification::customer_identification_failed(user, payload.reason),
//         notification::service::Backend::Email,
//     )
//     .await;
//
//     StatusCode::OK
// }

pub async fn dedicated_account_assignment_successful(
    ctx: Arc<types::Context>,
    payload: model::DedicatedAccountAssignmentSuccessful,
) -> impl IntoResponse {
    let user =
        match user::repository::find_by_email(&ctx.db_conn.pool, payload.customer.email).await {
            Ok(Some(user)) => user,
            _ => return StatusCode::INTERNAL_SERVER_ERROR,
        };

    match wallet::repository::update_metatata_by_owner_id(
        &ctx.db_conn.pool,
        user.id.clone(),
        wallet::repository::WalletMetadata {
            backend: Some(wallet::repository::WalletBackend::Paystack(
                wallet::repository::PaystackWalletMetadata {
                    customer: wallet::repository::PaystackCustomer {
                        id: payload.customer.id,
                        code: payload.customer.code,
                    },
                    dedicated_account: wallet::repository::PaystackDedicatedAccount {
                        id: payload.dedicated_account.id,
                        bank: wallet::repository::PaystackBank {
                            id: payload.dedicated_account.bank.id,
                            name: payload.dedicated_account.bank.name,
                            slug: payload.dedicated_account.bank.slug,
                        },
                        account_name: payload.dedicated_account.account_name,
                        account_number: payload.dedicated_account.account_number,
                        active: payload.dedicated_account.active,
                    },
                },
            )),
        },
    )
    .await
    {
        Ok(_) => (),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let _ = tokio::spawn(notification::service::send(
        ctx.clone(),
        notification::service::Notification::bank_account_creation_successful(user),
        notification::service::Backend::Email,
    ));

    StatusCode::OK
}

pub async fn dedicated_account_assignment_failed(
    ctx: Arc<types::Context>,
    payload: model::DedicatedAccountAssignmentFailed,
) -> impl IntoResponse {
    let user =
        match user::repository::find_by_email(&ctx.db_conn.pool, payload.customer.email.clone())
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                return StatusCode::NOT_FOUND;
            }
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

    let _ = notification::service::send(
        ctx.clone(),
        notification::service::Notification::bank_account_creation_failed(user),
        notification::service::Backend::Email,
    )
    .await;

    StatusCode::OK
}
