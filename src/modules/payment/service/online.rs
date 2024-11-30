use bigdecimal::{BigDecimal, FromPrimitive};
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    modules::{order::repository::Order, user::repository::User},
    types::Context,
};
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub order_id: String,
}

#[derive(Deserialize)]
pub struct PaystackResponseData {
    pub authorization_url: String,
}

#[derive(Deserialize)]
pub struct PaystackResponse {
    pub status: bool,
    pub data: PaystackResponseData,
}

pub struct InitializePaymentForOrder {
    pub order: Order,
    pub payer: User,
}

async fn create_payment_link(
    ctx: Arc<Context>,
    payload: InitializePaymentForOrder,
) -> Result<String, Error> {
    // TODO: probably get the meals so they can appear in the line items of the payment request

    let metadata = Metadata {
        order_id: payload.order.id.clone(),
    };

    let payload = json!({
        "email": payload.payer.email,
        "amount": payload.order.total * BigDecimal::from_u8(100).expect("Invalid primitive value to convert from"),
        "metadata": metadata,
    })
    .to_string();

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
        .post("https://api.paystack.co/transaction/initialize")
        .headers(headers)
        .body(payload.clone())
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to create payment link: {}", err);
            Error::UnexpectedError
        })?;

    if res.status() != StatusCode::OK {
        let data = res.text().await.map_err(|err| {
            tracing::error!(
                "Failed to process create payment link response for payload {}: {:?}",
                payload,
                err
            );
            Error::UnexpectedError
        })?;

        tracing::error!("Failed to create payment link: {}", data);
        return Err(Error::UnexpectedError);
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!(
            "Failed to process create payment link response for payload {}: {:?}",
            payload.clone(),
            err
        );
        Error::UnexpectedError
    })?;

    tracing::debug!("Response received from paystack server: {}", data);

    let paystack_response = serde_json::de::from_str::<PaystackResponse>(data.as_str())
        .map_err(|_| Error::UnexpectedError)?;

    if !paystack_response.status {
        tracing::error!("Failed to create payment link: {}", data);
        return Err(Error::UnexpectedError);
    }

    Ok(paystack_response.data.authorization_url)
}

pub async fn initialize_payment_for_order(
    ctx: Arc<Context>,
    payload: InitializePaymentForOrder,
) -> Result<serde_json::Value, Error> {
    let payment_link = create_payment_link(ctx.clone(), payload).await?;

    Ok(json!({ "url": payment_link }))
}
