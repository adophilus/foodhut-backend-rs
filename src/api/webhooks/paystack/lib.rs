pub mod handler {
    use super::super::model;
    use axum::response::IntoResponse;
    use bigdecimal::{BigDecimal, FromPrimitive};
    use hyper::StatusCode;
    use std::sync::Arc;

    use crate::{
        repository::{self, wallet},
        types, utils,
    };

    pub async fn successful_transaction(
        ctx: Arc<types::Context>,
        amount: BigDecimal,
        metadata: utils::online::Metadata,
    ) -> impl IntoResponse {
        let order =
            match repository::order::find_by_id(ctx.db_conn.clone(), metadata.order_id).await {
                Ok(Some(order)) => order,
                Ok(None) => return StatusCode::NOT_FOUND.into_response(),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

        if amount / BigDecimal::from_u8(100).expect("Invalid primitive value to convert from")
            < order.total
        {
            return StatusCode::BAD_REQUEST.into_response();
        }

        let cart =
            match repository::cart::find_by_id(ctx.db_conn.clone(), order.cart_id.clone()).await {
                Ok(Some(cart)) => cart,
                Ok(None) => return StatusCode::NOT_FOUND.into_response(),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

        if let Err(_) = repository::transaction::create(
            ctx.db_conn.clone(),
            repository::transaction::CreatePayload::Online(
                repository::transaction::CreateOnlineTransactionPayload {
                    amount: order.total.clone(),
                    r#type: repository::transaction::TransactionType::Debit,
                    note: Some(format!("Paid for order {}", order.id.clone())),
                    user_id: cart.owner_id.clone(),
                },
            ),
        )
        .await
        {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Err(_) = utils::payment::confirm_payment_for_order(ctx.clone(), order).await {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        StatusCode::OK.into_response()
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
    //     match utils::wallet::request_bank_account_creation(ctx.clone(), user.clone()).await {
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
    //     let _ = utils::notification::send(
    //         ctx.clone(),
    //         utils::notification::Notification::customer_identification_failed(user, payload.reason),
    //         utils::notification::Backend::Email,
    //     )
    //     .await;
    //
    //     StatusCode::OK
    // }

    pub async fn dedicated_account_assignment_successful(
        ctx: Arc<types::Context>,
        payload: model::DedicatedAccountAssignmentSuccessful,
    ) -> impl IntoResponse {
        let user = match repository::user::find_by_email(
            ctx.db_conn.clone(),
            payload.customer.email,
        )
        .await
        {
            Ok(Some(user)) => user,
            _ => return StatusCode::INTERNAL_SERVER_ERROR,
        };

        match wallet::update_metatata_by_owner_id(
            ctx.db_conn.clone(),
            user.id.clone(),
            wallet::WalletMetadata {
                backend: Some(wallet::WalletBackend::Paystack(
                    wallet::PaystackWalletMetadata {
                        customer: wallet::PaystackCustomer {
                            id: payload.customer.id,
                            code: payload.customer.code,
                        },
                        dedicated_account: wallet::PaystackDedicatedAccount {
                            id: payload.dedicated_account.id,
                            bank: wallet::PaystackBank {
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

        let _ = tokio::spawn(utils::notification::send(
            ctx.clone(),
            utils::notification::Notification::bank_account_creation_successful(user),
            utils::notification::Backend::Email,
        ));

        StatusCode::OK
    }

    pub async fn dedicated_account_assignment_failed(
        ctx: Arc<types::Context>,
        payload: model::DedicatedAccountAssignmentFailed,
    ) -> impl IntoResponse {
        let user = match repository::user::find_by_email(
            ctx.db_conn.clone(),
            payload.customer.email.clone(),
        )
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

        let _ = utils::notification::send(
            ctx.clone(),
            utils::notification::Notification::bank_account_creation_failed(user),
            utils::notification::Backend::Email,
        )
        .await;

        StatusCode::OK
    }
}
