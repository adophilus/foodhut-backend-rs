use super::{types, Error, Notification, Result};
use crate::types::{AppEnvironment, Context};
use axum::http::HeaderMap;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct OtpSmsServerResponse {
    #[serde(rename = "pinId")]
    pin_id: String,
    status: String,
}

pub struct Otp {
    pub pin_id: String,
}

pub async fn send(ctx: Arc<Context>, notification: Notification) -> Result<Otp> {
    match notification.clone() {
        Notification::VerificationOtpRequested(n) => send_verification_otp(ctx, n).await,
        _ => Err(Error::InvalidNotification),
    }
}

async fn send_verification_otp(
    ctx: Arc<Context>,
    _notification: types::VerificationOtpRequested,
) -> Result<Otp> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json"
            .try_into()
            .expect("Invalid content type header value"),
    );

    let validity = match ctx.app.environment {
        AppEnvironment::Production => 5,
        AppEnvironment::Development => 3,
    };

    tracing::debug!("validity: {}", validity);

    let res = reqwest::Client::new()
                .post(ctx.otp.send_endpoint.clone())
        .headers(headers)
                .body(json!({
                    "api_key": ctx.otp.api_key.clone(),
                    "message_type": "NUMERIC",
                    "to": _notification.user.phone_number.clone(),
                    "from":ctx.otp.app_id.clone(),
                    "channel": "dnd",
                    "pin_attempts": 3,
                    "pin_time_to_live": validity,
                    "pin_length": 4,
                    "pin_placeholder":"$PIN",
                    "message_text": format!("Your FoodHut app verification pin is $PIN\nPin will expire in {} minutes, do not share it with anyone", validity),
                    "pin_type":  "NUMERIC"
                }).to_string())
                .send()
                .await
    .map_err(|err| {
            tracing::error!("Failed to send OTP sms: {}", err);
        Error::NotSent
})?;

    if res.status() != StatusCode::OK {
        match res.text().await {
            Ok(data) => {
                let formatted_err = format!("Failed to send OTP sms: {}", data);
                tracing::error!(formatted_err);
                return Err(Error::NotSent);
            }
            Err(err) => {
                let formatted_err = format!("Failed to get response body: {}", err);
                tracing::error!(formatted_err);
                return Err(Error::NotSent);
            }
        }
    }

    let res_text = res.text().await.map_err(|err| {
        let formatted_err = format!("Failed to get response body: {}", err);
        tracing::error!(formatted_err);
        Error::NotSent
    })?;

    tracing::info!("res_text: {}", res_text);

    let res = serde_json::from_str::<OtpSmsServerResponse>(&res_text)
        .map_err(|err| {
            let formatted_err = format!("Failed to get response body: {}", err);
            tracing::error!(formatted_err);
            Error::NotSent
        })
        .map_err(|_| Error::NotSent)?;

    if res.status != "200" {
        tracing::error!("Got an unexpected status response: {}", res_text);
        return Err(Error::NotSent);
    }

    tracing::debug!("Successfully sent OTP sms");
    tracing::debug!("PIN ID: {}", res.pin_id.clone());

    return Ok(Otp { pin_id: res.pin_id });
}
