use super::repository::{self, Wallet};
use axum::http::HeaderMap;
use axum::http::Method;
use bigdecimal::BigDecimal;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_string_from_number;
use serde_json::json;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;

use crate::{
    modules::{
        kitchen, order::repository::Order, payment, transaction, user::repository::User, wallet,
    },
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
    match payment::utils::send_paystack_request::<CustomerCreationServiceResponse>(
        ctx.clone(),
        payment::utils::SendPaystackRequestPayload {
            body: Some(
                json!({
                    "email": owner.email,
                    "first_name": owner.first_name,
                    "last_name": owner.last_name,
                    "phone": owner.phone_number,
                })
                .to_string(),
            ),
            method: Method::POST,
            route: String::from("/customer"),
            expected_status_code: StatusCode::OK,
            query: None,
        },
    )
    .await
    {
        Ok(res) => {
            if !res.status {
                return Err(CreationError::CreationFailed(res.message));
            }

            Ok(())
        }
        _ => Err(CreationError::UnexpectedError),
    }
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

    match payment::utils::send_paystack_request::<InitializeBankAccountCreationServiceResponse>(
        ctx.clone(),
        payment::utils::SendPaystackRequestPayload {
            expected_status_code: StatusCode::OK,
            route: String::from("/dedicated_account/assign"),
            method: Method::POST,
            body: Some(
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
            ),
            query: None
        },
    )
    .await
    {
        Ok(res) => {
            if !res.status {
                return Err(CreationError::CreationFailed(res.message));
            }

            Ok(res.message)
        }
        _ => Err(CreationError::UnexpectedError),
    }
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
    pub as_kitchen: bool,
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

    let wallet = if payload.as_kitchen {
        let kitchen = kitchen::repository::find_by_owner_id(&mut *tx, payload.user.id.clone())
            .await
            .map_err(|_| WithdrawFundsError::UnexpectedError)?
            .ok_or(WithdrawFundsError::UnexpectedError)?;
        repository::find_by_kitchen_id(&mut *tx, kitchen.id).await
    } else {
        repository::find_by_owner_id(&mut *tx, payload.user.id.clone()).await
    }
    .map_err(|_| WithdrawFundsError::UnexpectedError)?
    .ok_or(WithdrawFundsError::UnexpectedError)?;

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

#[derive(Debug, Serialize, Deserialize)]
struct PaystackBank {
    id: u64,
    name: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct PaystackBankResponse {
    status: bool,
    data: Vec<PaystackBank>,
}

pub enum PaystackBankError {
    UnexpectedError,
}

async fn fetch_paystack_banks(
    ctx: Arc<Context>,
) -> std::result::Result<Vec<PaystackBank>, PaystackBankError> {
    match payment::utils::send_paystack_request::<PaystackBankResponse>(
        ctx.clone(),
        payment::utils::SendPaystackRequestPayload {
            route: String::from("/bank"),
            method: Method::GET,
            body: None,
            expected_status_code: StatusCode::OK,
            query: Some(&[("country", "nigeria"), ("perPage", "100")]),
        },
    )
    .await
    {
        Ok(res) => {
            if !res.status {
                tracing::error!("Failed to fetch paystack banks, false status: {:?}", res.data);
                return Err(PaystackBankError::UnexpectedError);
            }
            Ok(res.data)
        }
        _ => Err(PaystackBankError::UnexpectedError),
    }
}

pub async fn update_paystack_banks(
    ctx: Arc<Context>,
) -> std::result::Result<(), PaystackBankError> {
    let banks = fetch_paystack_banks(ctx.clone()).await?;

    repository::update_banks_batch(
        &ctx.db_conn.pool,
        banks
            .into_iter()
            .map(|bank| repository::DbPaystackBankUpdate {
                id: bank.id.to_string(),
                name: bank.name,
                code: bank.code,
            })
            .collect::<Vec<_>>(),
    )
    .await;

    Ok(())
}

#[derive(Deserialize)]
pub struct GetBankAccountDetailsPayload {
    pub account_number: String,
    pub bank_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct BankAccountDetails {
    pub account_name: String,
}

#[derive(Deserialize)]
struct PayastackBankAccountDetailsResponse {
    status: bool,
    data: BankAccountDetails,
}

pub async fn get_bank_account_details(
    ctx: Arc<Context>,
    payload: GetBankAccountDetailsPayload,
) -> std::result::Result<BankAccountDetails, PaystackBankError> {
    let mut headers = HeaderMap::new();
    let auth_header = format!("Bearer {}", ctx.payment.secret_key);
    headers.insert(
        "Authorization",
        auth_header
            .clone()
            .try_into()
            .expect("Invalid auth header value"),
    );
    headers.insert(
        "Content-Type",
        "application/json"
            .try_into()
            .expect("Invalid content type header value"),
    );

    let res = reqwest::Client::new()
        .get(format!("{}/bank/resolve", ctx.payment.api_endpoint))
        .headers(headers.clone())
        .query(&[
            ("account_number", payload.account_number),
            ("bank_code", payload.bank_code),
        ])
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch paystack banks: {}", err);
            PaystackBankError::UnexpectedError
        })?;

    if res.status() != StatusCode::OK {
        let status = res.status();
        let data = res.text().await.map_err(|err| {
            tracing::error!("Failed to process fetch banks response: {:?}", err);
            PaystackBankError::UnexpectedError
        })?;

        tracing::error!(
            "Failed to fetch paystack banks invalid status code {}: {}",
            status,
            data
        );
        return Err(PaystackBankError::UnexpectedError);
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!(
            "Failed to process fetch paystack bank account details response: {:?}",
            err
        );
        PaystackBankError::UnexpectedError
    })?;

    let paystack_response =
        serde_json::de::from_str::<PayastackBankAccountDetailsResponse>(data.as_str())
            .map_err(|_| PaystackBankError::UnexpectedError)?;

    if !paystack_response.status {
        tracing::error!(
            "Failed to fetch paystack bank account defailt, false status: {}",
            data
        );
        return Err(PaystackBankError::UnexpectedError);
    }

    Ok(paystack_response.data)
}
