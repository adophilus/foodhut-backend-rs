use axum::http::{HeaderMap, HeaderValue};
use bigdecimal::{BigDecimal, FromPrimitive};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    repository::{
        meal::Meal,
        order::{self, Order},
        transaction,
        user::User,
    },
    types::Context,
    utils::database::DatabaseConnection,
};
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
}

pub struct InitializePaymentForOrder {
    pub order: Order,
    pub payer: User,
}

async fn create_payment_link(
    ctx: Arc<Context>,
    payload: InitializePaymentForOrder,
) -> Result<String, Error> {
    let meals = match order::get_meals_from_order_by_id(
        ctx.db_conn.clone(),
        payload.order.id.clone(),
    )
    .await
    {
        Ok(meals) => meals,
        Err(_) => return Err(Error::UnexpectedError),
    };

    let payload = json!({
        "email": payload.payer.email,
        "amount": payload.order.total * BigDecimal::from_u8(100).expect("Invalid primitive value to convert from"),
        "metatadata": {
            "order_id": payload.order.id.clone()
        }
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

    Ok(String::new())
}

pub async fn initialize_payment_for_order(
    ctx: Arc<Context>,
    payload: InitializePaymentForOrder,
) -> Result<(), Error> {
    let payment_link = create_payment_link(ctx.clone(), payload).await?;

    tracing::debug!("Payment link: {}", payment_link);

    Ok(())
}
