use super::super::{Error, Notification, Result};
use crate::types;
use crate::utils::notification;
use axum::http::{HeaderMap, HeaderValue};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct OtpSmsServerResponse {
    pin_id: String,
    status: String,
}

pub struct Otp {
    pub pin_id: String,
}

pub async fn send(ctx: Arc<types::Context>, notification: Notification) -> Result<Otp> {
    match notification.clone() {
        Notification::Registered(n) => unimplemented!(),
        Notification::OrderPaid(n) => unimplemented!(),
        notification::Notification::PasswordResetRequested(n) => unimplemented!(),
        notification::Notification::VerificationOtpRequested(n) => {
            send_verification_otp(ctx, n).await
        }
    }
}

async fn send_verification_otp(
    ctx: Arc<types::Context>,
    _notification: notification::types::VerificationOtpRequested,
) -> Result<Otp> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json"
            .try_into()
            .expect("Invalid content type header value"),
    );

    let res = reqwest::Client::new()
                .post(ctx.otp.send_endpoint.clone())
        .headers(headers)
                .body(json!({
                    "api_key": ctx.otp.api_key.clone(),
                    "message_type": "NUMERIC",
                    "to": _notification.user.phone_number.clone(),
                    "from":ctx.otp.app_id.clone(),
                    "channel":"generic",
                    "pin_attempts": 3,
                    "pin_time_to_live": 5,
                    "pin_length": 4,
                    "pin_placeholder":"$PIN",
                    "message_text": "Your FoodHut app verification pin is $PIN\nPin will expire in 5 minutes, do not share it with anyone" ,
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

    let res = res
        .text()
        .await
        .map_err(|err| {
            let formatted_err = format!("Failed to get response body: {}", err);
            tracing::error!(formatted_err);
            Error::NotSent
        })
        .and_then(|data| {
            serde_json::from_str::<OtpSmsServerResponse>(&data).map_err(|err| {
                let formatted_err = format!("Failed to get response body: {}", err);
                tracing::error!(formatted_err);
                Error::NotSent
            })
        })
        .map_err(|_| Error::NotSent)?;

    tracing::debug!("Successfully sent OTP sms");

    return Ok(Otp { pin_id: res.pin_id });
}
