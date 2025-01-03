use super::repository::{self, Wallet};
use axum::http::HeaderMap;
use bigdecimal::BigDecimal;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_string_from_number;
use serde_json::json;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;

use crate::{
    modules::{order::repository::Order, payment, transaction, user::repository::User, wallet},
    types::AppEnvironment,
    Context,
};

pub enum Error {
    UnexpectedError,
    WalletNotFound,
    InsufficientFunds,
}

pub enum CreationError {
    CreationFailed(String),
    UnexpectedError,
}

#[derive(Serialize, Deserialize)]
struct CustomerCreationServiceResponseData {
    #[serde(deserialize_with = "deserialize_string_from_number")]
    id: String,
    customer_code: String,
}

#[derive(Serialize, Deserialize)]
struct CustomerCreationServiceResponse {
    status: bool,
    message: String,
    data: CustomerCreationServiceResponseData,
}

pub async fn create(ctx: Arc<Context>, owner: User) -> std::result::Result<(), CreationError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", ctx.payment.secret_key.clone())
            .try_into()
            .expect("Invalid authorization header value"),
    );

    let res = reqwest::Client::new()
        .post("https://api.paystack.co/customer")
        .headers(headers)
        .body(
            json!({
                "email": owner.email,
                "first_name": owner.first_name,
                "last_name": owner.last_name,
                "phone": owner.phone_number,
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to communicate with payment service: {}", err);
            CreationError::UnexpectedError
        })?;

    let status = res.status();
    if status != StatusCode::OK {
        tracing::error!("Got an error response from the payment service: {}", status);
        return Err(CreationError::UnexpectedError);
    }

    let text_response = res.text().await.map_err(|err| {
        tracing::error!("Failed to get payment service text response: {}", err);
        CreationError::UnexpectedError
    })?;

    let server_response = serde_json::from_str::<CustomerCreationServiceResponse>(
        text_response.as_str(),
    )
    .map_err(|err| {
        tracing::error!("Failed to decode payment service server response: {}", err);
        CreationError::UnexpectedError
    })?;

    if !server_response.status {
        return Err(CreationError::CreationFailed(server_response.message));
    }

    Ok(())

    // wallet::create(
    //     e,
    //     wallet::CreateWalletPayload {
    //         owner_id: owner.id.clone(),
    //         metadata: wallet::WalletMetadata {
    //             backend: WalletBackend::Paystack(wallet::PaystackWalletMetadata {
    //                 customer: wallet::PaystackCustomer {
    //                     id: server_response.data.id.clone(),
    //                     code: server_response.data.customer_code.clone(),
    //                 },
    //                 dedicated_account: None,
    //             }),
    //         },
    //     },
    // )
    // .await
    // .map_err(|_| CreationError::UnexpectedError)
}

pub struct RequestVirtualAccountPayload {
    pub bvn: String,
    pub bank_code: String,
    pub account_number: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeBankAccountCreationServiceResponse {
    status: bool,
    message: String,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn request_virtual_account(
    ctx: Arc<Context>,
    payload: RequestVirtualAccountPayload,
) -> std::result::Result<String, CreationError> {
    let _wallet = wallet::repository::find_by_owner_id(&ctx.db_conn.pool, payload.user.id.clone())
        .await
        .map_err(|_| CreationError::UnexpectedError)?
        .ok_or(CreationError::UnexpectedError)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", ctx.payment.secret_key.clone())
            .try_into()
            .expect("Invalid authorization header value"),
    );

    let preferred_bank = match ctx.app.environment {
        AppEnvironment::Production => "titan-paystack",
        AppEnvironment::Development => "test-bank",
    };

    let res = reqwest::Client::new()
        .post(format!(
            "{}/dedicated_account/assign",
            ctx.payment.api_endpoint.clone(),
        ))
        .headers(headers)
        .body(
            json!({
                "country": "NG",
                "type": "bank_account",
                "account_number": payload.account_number,
                "bvn": payload.bvn,
                "bank_code": payload.bank_code,
                "first_name": payload.user.first_name,
                "last_name": payload.user.last_name,
                "email": payload.user.email,
                "phone": payload.user.phone_number,
                "preferred_bank": preferred_bank,
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to communicate with payment service: {}", err);
            CreationError::UnexpectedError
        })?;

    let status = res.status();
    if status != StatusCode::OK {
        tracing::warn!("Got an error response from the payment service: {}", status);
    }

    let text_response = res.text().await.map_err(|err| {
        tracing::error!("Failed to get payment service text response: {}", err);
        CreationError::UnexpectedError
    })?;

    tracing::info!("{}", text_response);

    let server_response = serde_json::from_str::<InitializeBankAccountCreationServiceResponse>(
        text_response.as_str(),
    )
    .map_err(|err| {
        tracing::error!("Failed to decode payment service server response: {}", err);
        CreationError::UnexpectedError
    })?;

    tracing::info!("{:?}", server_response);

    if !server_response.status {
        return Err(CreationError::CreationFailed(server_response.message));
    }

    Ok(server_response.message)
}

#[derive(Serialize, Deserialize)]
pub struct RequestBankAccountCreationServiceResponse {
    status: bool,
    message: String,
}

// pub async fn request_bank_account_creation(
//     ctx: Arc<Context>,
//     user: User,
// ) -> std::result::Result<String, CreationError> {
//     let _wallet = wallet::find_by_owner_id(ctx.db_conn.clone(), user.id.clone())
//         .await
//         .map_err(|_| CreationError::UnexpectedError)?
//         .ok_or(CreationError::UnexpectedError)?;
//     let mut headers = HeaderMap::new();
//     headers.insert(
//         "Authorization",
//         format!("Bearer {}", ctx.payment.secret_key.clone())
//             .try_into()
//             .expect("Invalid authorization header value"),
//     );
//
//     let customer_code = match _wallet.metadata.backend {
//         WalletBackend::Paystack(backend) => backend.customer.code.clone(),
//     };
//
//     let res = reqwest::Client::new()
//         .post(format!(
//             "{}/customer/{}/identification",
//             ctx.payment.api_endpoint.clone(),
//             customer_code.clone()
//         ))
//         .headers(headers)
//         .body(
//             json!({
//                 "customer": customer_code.clone(),
//                 "preferred_bank": "titan-paystack",
//             })
//             .to_string(),
//         )
//         .send()
//         .await
//         .map_err(|err| {
//             tracing::error!("Failed to communicate with payment service: {}", err);
//             CreationError::UnexpectedError
//         })?;
//
//     let status = res.status();
//     if status != StatusCode::OK {
//         tracing::error!("Got an error response from the payment service: {}", status);
//         return Err(CreationError::UnexpectedError);
//     }
//
//     let text_response = res.text().await.map_err(|err| {
//         tracing::error!("Failed to get payment service text response: {}", err);
//         CreationError::UnexpectedError
//     })?;
//
//     let server_response =
//         serde_json::from_str::<RequestBankAccountCreationServiceResponse>(text_response.as_str())
//             .map_err(|err| {
//             tracing::error!("Failed to decode payment service server response: {}", err);
//             CreationError::UnexpectedError
//         })?;
//
//     if !server_response.status {
//         return Err(CreationError::CreationFailed(server_response.message));
//     }
//
//     Ok(server_response.message)
// }

pub struct InitializePaymentForOrder {
    pub order: Order,
    pub payer: User,
}

pub async fn initialize_payment_for_order(
    ctx: Arc<Context>,
    tx: &mut Transaction<'_, Postgres>,
    payload: InitializePaymentForOrder,
) -> Result<()> {
    let wallet = wallet::repository::find_by_owner_id(&mut **tx, payload.payer.id.clone())
        .await
        .map_err(|_| Error::UnexpectedError)?
        .ok_or(Error::WalletNotFound)?;

    if wallet.balance < payload.order.total {
        return Err(Error::InsufficientFunds);
    }

    payment::service::confirm_payment_for_order(
        ctx,
        tx,
        payment::service::ConfirmPaymentForOrderPayload {
            payment_method: payment::service::PaymentMethod::Wallet,
            order: payload.order,
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    Ok(())
}

pub struct ConfirmPaymentForOrderPayload {
    pub order: Order,
    pub wallet: Wallet,
}

pub async fn confirm_payment(
    tx: &mut Transaction<'_, Postgres>,
    payload: ConfirmPaymentForOrderPayload,
) -> Result<()> {
    transaction::repository::create(
        &mut **tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: payload.order.total.clone(),
                direction: transaction::repository::TransactionDirection::Outgoing,
                note: Some(format!("Paid for order {}", payload.order.id.clone())),
                wallet_id: payload.wallet.id.clone(),
                user_id: payload.wallet.owner_id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    repository::update_by_id(
        &mut **tx,
        payload.wallet.id.clone(),
        repository::UpdateByIdPayload {
            operation: repository::UpdateOperation::Debit,
            amount: payload.order.total.clone(),
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)
}

pub struct CreateTopupInvoicePayload {
    pub user: User,
    pub amount: BigDecimal,
}

pub async fn create_topup_invoice(
    ctx: Arc<Context>,
    payload: CreateTopupInvoicePayload,
) -> Result<String> {
    payment::service::online::create_topup_invoice(
        ctx,
        payment::service::online::CreateTopupInvoicePayload {
            user: payload.user.clone(),
            amount: payload.amount.clone(),
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)
}

pub struct WithdrawFundsPayload {
    pub account_number: String,
    pub bank_code: String,
    pub account_name: String,
    pub amount: BigDecimal,
    pub user: User,
}

pub enum WithdrawFundsError {
    InsufficientBalance,
    UnexpectedError,
}

pub async fn withdraw_funds(
    ctx: Arc<Context>,
    payload: WithdrawFundsPayload,
) -> std::result::Result<(), WithdrawFundsError> {
    let mut tx = ctx
        .db_conn
        .pool
        .begin()
        .await
        .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    let maybe_wallet = repository::find_by_owner_id(&mut *tx, payload.user.id.clone())
        .await
        .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    let wallet = match maybe_wallet {
        Some(wallet) => wallet,
        None => return Err(WithdrawFundsError::UnexpectedError),
    };

    if wallet.balance < payload.amount {
        return Err(WithdrawFundsError::InsufficientBalance);
    }

    payment::service::online::withdraw_funds(
        ctx.clone(),
        payment::service::online::WithdrawFundsPayload {
            account_name: payload.account_name.clone(),
            account_number: payload.account_number.clone(),
            amount: payload.amount.clone(),
            bank_code: payload.bank_code.clone(),
            user: payload.user.clone(),
        },
    )
    .await
    .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    transaction::repository::create(
        &mut *tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: payload.amount.clone(),
                direction: transaction::repository::TransactionDirection::Outgoing,
                note: Some(format!(
                    "Withdrawal to {} {}",
                    payload.account_name, payload.account_number
                )),
                wallet_id: wallet.id.clone(),
                user_id: payload.user.id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    repository::update_by_id(
        &mut *tx,
        wallet.id.clone(),
        repository::UpdateByIdPayload {
            operation: repository::UpdateOperation::Debit,
            amount: payload.amount.clone(),
        },
    )
    .await
    .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    tx.commit()
        .await
        .map_err(|_| WithdrawFundsError::UnexpectedError)?;

    Ok(())
}
