use axum::http::HeaderMap;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_string_from_number;
use serde_json::json;
use sqlx::PgExecutor;
use std::sync::Arc;

use crate::{
    repository::{
        self,
        meal::Meal,
        order::{self, Order},
        transaction,
        user::User,
        wallet::{self, WalletBackend},
    },
    types::{self, AppEnvironment},
    utils::database::DatabaseConnection,
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

pub async fn create<'e, E>(
    ctx: Arc<types::Context>,
    e: E,
    owner: repository::user::User,
) -> std::result::Result<(), CreationError>
where
    E: PgExecutor<'e>,
{
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
    ctx: Arc<types::Context>,
    payload: RequestVirtualAccountPayload,
) -> std::result::Result<String, CreationError> {
    let _wallet = wallet::find_by_owner_id(ctx.db_conn.clone(), payload.user.id.clone())
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
//     ctx: Arc<types::Context>,
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

struct InitializePaymentForMeal {
    payer: User,
    meal: Meal,
}

pub struct InitializePaymentForOrder {
    pub order: Order,
    pub payer: User,
}

pub async fn initialize_payment_for_order(
    db: DatabaseConnection,
    payload: InitializePaymentForOrder,
) -> Result<()> {
    let wallet = match wallet::find_by_owner_id(db.clone(), payload.payer.id.clone()).await {
        Ok(Some(wallet)) => wallet,
        Ok(None) => return Err(Error::WalletNotFound),
        Err(_) => return Err(Error::UnexpectedError),
    };

    if wallet.balance < payload.order.total {
        return Err(Error::InsufficientFunds);
    }

    if let Err(_) = wallet::update_by_id(
        db.clone(),
        wallet.id.clone(),
        wallet::UpdateWalletPayload {
            operation: wallet::UpdateWalletOperation::Debit,
            amount: payload.order.total.clone(),
        },
    )
    .await
    {
        return Err(Error::UnexpectedError);
    };

    wallet::update_by_id(
        db.clone(),
        wallet.id.clone(),
        wallet::UpdateWalletPayload {
            operation: wallet::UpdateWalletOperation::Debit,
            amount: payload.order.total.clone(),
        },
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    transaction::create(
        db.clone(),
        transaction::CreatePayload::Wallet(transaction::CreateWalletTransactionPayload {
            amount: payload.order.total.clone(),
            r#type: transaction::TransactionType::Debit,
            note: Some(format!("Paid for order {}", payload.order.id.clone())),
            wallet_id: wallet.id.clone(),
        }),
    )
    .await
    .map_err(|_| Error::UnexpectedError)?;

    Ok(())
}
