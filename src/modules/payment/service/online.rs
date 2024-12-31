use bigdecimal::BigDecimal;
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
pub struct OrderInvoiceMetadata {
    pub order_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TopupMetadata {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    Order(OrderInvoiceMetadata),
    Topup(TopupMetadata),
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

async fn create_paystack_invoice(ctx: Arc<Context>, payload: String) -> Result<String, Error> {
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
                "Failed to process create payment response for payload {}: {:?}",
                payload,
                err
            );
            Error::UnexpectedError
        })?;

        tracing::error!("Failed to create invoice link: {}", data);
        return Err(Error::UnexpectedError);
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!(
            "Failed to process create invoice link response for payload {}: {:?}",
            payload.clone(),
            err
        );
        Error::UnexpectedError
    })?;

    tracing::debug!("Response received from paystack server: {}", data);

    let paystack_response = serde_json::de::from_str::<PaystackResponse>(data.as_str())
        .map_err(|_| Error::UnexpectedError)?;

    if !paystack_response.status {
        tracing::error!("Failed to create invoice link: {}", data);
        return Err(Error::UnexpectedError);
    }

    Ok(paystack_response.data.authorization_url)
}

pub struct InitializeInvoiceForOrder {
    pub order: Order,
    pub payer: User,
}

async fn create_invoice_link_for_order(
    ctx: Arc<Context>,
    payload: InitializeInvoiceForOrder,
) -> Result<String, Error> {
    // TODO: probably get the meals so they can appear in the line items of the payment request

    let metadata = OrderInvoiceMetadata {
        order_id: payload.order.id.clone(),
    };

    let payload = json!({
        "email": payload.payer.email,
        "amount": payload.order.total * BigDecimal::from(100),
        "metadata": metadata,
    })
    .to_string();

    create_paystack_invoice(ctx, payload).await
}

pub async fn initialize_invoice_for_order(
    ctx: Arc<Context>,
    payload: InitializeInvoiceForOrder,
) -> Result<serde_json::Value, Error> {
    let payment_link = create_invoice_link_for_order(ctx.clone(), payload).await?;

    Ok(json!({ "url": payment_link }))
}

pub struct CreateTopupInvoicePayload {
    pub amount: BigDecimal,
    pub user: User,
}

pub async fn create_topup_invoice(
    ctx: Arc<Context>,
    payload: CreateTopupInvoicePayload,
) -> Result<String, Error> {
    let metadata = TopupMetadata {
        user_id: payload.user.id.clone(),
    };

    let payload = json!({
        "email": payload.user.email.clone(),
        "amount": payload.amount * BigDecimal::from(100),
        "metadata": metadata,
    })
    .to_string();

    create_paystack_invoice(ctx, payload).await
}
