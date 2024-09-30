use axum::http::HeaderMap;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::{
    repository::{
        meal::Meal,
        order::{self, Order},
        transaction,
        user::User,
        wallet::{self, WalletBackend},
    },
    types,
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

pub struct RequestBankAccountVerificationPayload {
    pub bvn: String,
    pub bank_code: String,
    pub account_number: String,
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct InitializeBankAccountCreationServiceResponse {
    status: bool,
    message: String,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn request_bank_account_verification(
    ctx: Arc<types::Context>,
    payload: RequestBankAccountVerificationPayload,
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

    let customer_code = match _wallet.metadata.backend {
        WalletBackend::Paystack(backend) => backend.customer.code.clone(),
    };

    let res = reqwest::Client::new()
        .post(format!(
            "https://api.paystack.co/customer/{}/identification",
            customer_code
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

    let server_response = serde_json::from_str::<InitializeBankAccountCreationServiceResponse>(
        text_response.as_str(),
    )
    .map_err(|err| {
        tracing::error!("Failed to decode payment service server response: {}", err);
        CreationError::UnexpectedError
    })?;

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

pub async fn request_bank_account_creation(
    ctx: Arc<types::Context>,
    user: User,
) -> std::result::Result<String, CreationError> {
    let _wallet = wallet::find_by_owner_id(ctx.db_conn.clone(), user.id.clone())
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

    let customer_code = match _wallet.metadata.backend {
        WalletBackend::Paystack(backend) => backend.customer.code.clone(),
    };

    let res = reqwest::Client::new()
        .post(format!(
            "https://api.paystack.co/customer/{}/identification",
            customer_code.clone()
        ))
        .headers(headers)
        .body(
            json!({
                "customer": customer_code.clone(),
                "preferred_bank": "titan-paystack",
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

    let server_response =
        serde_json::from_str::<RequestBankAccountCreationServiceResponse>(text_response.as_str())
            .map_err(|err| {
            tracing::error!("Failed to decode payment service server response: {}", err);
            CreationError::UnexpectedError
        })?;

    if !server_response.status {
        return Err(CreationError::CreationFailed(server_response.message));
    }

    Ok(server_response.message)
}

struct InitializePaymentForMeal {
    payer: User,
    meal: Meal,
}

async fn initialize_payment_for_meal(
    db: DatabaseConnection,
    payload: InitializePaymentForMeal,
) -> Result<()> {
    let wallet = match wallet::find_by_owner_id(db.clone(), payload.payer.id.clone()).await {
        Ok(Some(wallet)) => wallet,
        Ok(None) => return Err(Error::WalletNotFound),
        Err(_) => return Err(Error::UnexpectedError),
    };

    if wallet.balance < payload.meal.price {
        return Err(Error::InsufficientFunds);
    }

    // TODO: switch on the error, because it could just be an insufficient balance
    // TODO: also not sure which of these should come first
    // TODO: these operations need to be processed in a transaction

    if let Err(_) = wallet::update_by_id(
        db.clone(),
        wallet.id.clone(),
        wallet::UpdateWalletPayload {
            operation: wallet::UpdateWalletOperation::Debit,
            amount: payload.meal.price.clone(),
        },
    )
    .await
    {
        return Err(Error::UnexpectedError);
    };

    if let Err(_) = transaction::create(
        db.clone(),
        transaction::CreatePayload::Wallet(transaction::CreateWalletTransactionPayload {
            amount: payload.meal.price.clone(),
            r#type: transaction::TransactionType::Debit,
            note: Some(format!("Paid for {}", payload.meal.name.clone())),
            wallet_id: wallet.id.clone(),
        }),
    )
    .await
    {
        return Err(Error::UnexpectedError);
    };

    Ok(())
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

    let meals = match order::get_meals_from_order_by_id(db.clone(), payload.order.id).await {
        Ok(meals) => meals,
        Err(_) => return Err(Error::UnexpectedError),
    };

    for meal in meals {
        match initialize_payment_for_meal(
            db.clone(),
            InitializePaymentForMeal {
                meal,
                payer: payload.payer.clone(),
            },
        )
        .await
        {
            Ok(_) => (),
            Err(_) => return Err(Error::UnexpectedError),
        }
    }

    Ok(())
}
