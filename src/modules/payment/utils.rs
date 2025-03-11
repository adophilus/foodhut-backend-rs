use crate::types::Context;
use axum::http::{HeaderMap, Method, StatusCode};
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub struct SendPaystackRequestPayload<'a> {
    pub route: String,
    pub body: Option<String>,
    pub expected_status_code: StatusCode,
    pub method: Method,
    pub query: Option<&'a [(&'a str, &'a str)]>,
}

pub enum Error {
    RequestNotSent,
    InvalidHttpResponseStatusCode,
    FailedToDecodeResponse,
}

pub async fn send_paystack_request<'a, R: DeserializeOwned>(
    ctx: Arc<Context>,
    payload: SendPaystackRequestPayload<'a>,
) -> Result<R, Error> {
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

    let url = format!("{}{}", ctx.payment.api_endpoint, payload.route);
    let client = reqwest::Client::new();
    let mut req = match payload.method {
        Method::GET => client.get(url),
        _ => client.post(url),
    };

    match payload.query {
        Some(query) => req = req.query(query),
        _ => (),
    };

    req = req.headers(headers);

    match payload.body {
        Some(body) => {
            req = req.body(body);
        }
        None => (),
    };

    let res = req.send().await.map_err(|err| {
        tracing::error!("Failed to send Paystack request: {}", err);
        Error::RequestNotSent
    })?;

    let http_response_status_code = res.status();

    if http_response_status_code != payload.expected_status_code {
        tracing::error!(
            "Got unexpected http response status: {}",
            http_response_status_code
        );
        Err(Error::InvalidHttpResponseStatusCode)?
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!("Failed to get text of failed paystack request: {}", err);
        Error::InvalidHttpResponseStatusCode
    })?;

    tracing::trace!("Response received from paystack server: {}", data);

    let paystack_response = serde_json::de::from_str::<R>(&data).map_err(|err| {
        tracing::error!("Failed to decode Paystack response: {}", err);
        Error::FailedToDecodeResponse
    })?;

    Ok(paystack_response)
}
