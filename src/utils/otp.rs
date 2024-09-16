use chrono::Utc;

use hyper::HeaderMap;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use tracing_subscriber::util::SubscriberInitExt;

use super::notification;
use crate::{
    repository,
    types::{AppEnvironment, Context},
};
use std::sync::Arc;

pub enum SendError {
    NotSent,
    NotExpired,
    Expired,
}

pub enum VerificationError {
    Expired,
    InvalidOtp,
    UnexpectedError,
}

#[derive(Deserialize)]
struct VerificationEndpointPayload {
    verified: String,
    #[serde(rename = "pinId")]
    pin_id: String,
}

fn generate_hash(purpose: &str, user: &repository::user::User) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{}-{}", purpose.clone(), user.id.clone()));
    let hash = hasher.finalize();
    let hash = base16ct::lower::encode_string(&hash);
    tracing::debug!("hash: {}", hash.clone());
    hash
}

async fn hit_up_endpoint_and_parse(
    endpoint: String,
    body: String,
) -> Result<VerificationEndpointPayload, SendError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json"
            .try_into()
            .expect("Invalid content type header value"),
    );

    let res = Client::new()
        .post(endpoint)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to send request {}", err);
            SendError::NotSent
        })?;

    let text = res.text().await.map_err(|err| {
        tracing::error!("Failed to get response text {}", err);
        SendError::NotSent
    })?;

    serde_json::from_str::<_>(&text).map_err(|err| {
        tracing::error!("Failed to deserialize text: {}", err);
        SendError::NotSent
    })
}

pub async fn send(
    ctx: Arc<Context>,
    user: repository::user::User,
    purpose: String,
) -> Result<repository::otp::Otp, SendError> {
    let hash = generate_hash(&purpose, &user);

    let existing_otp = repository::otp::find_by_hash(ctx.db_conn.clone(), hash.clone())
        .await
        .map_err(|_| SendError::NotSent)?;

    if existing_otp.is_some() && Utc::now().naive_utc() <= existing_otp.clone().unwrap().expires_at
    {
        return Err(SendError::NotSent);
    }

    let otp = notification::sms::send(
        ctx.clone(),
        notification::Notification::verification_otp_requested(user.clone()),
    )
    .await
    .map_err(|_| SendError::NotSent)?;

    let validity = match ctx.app.environment {
        AppEnvironment::Production => 5,
        AppEnvironment::Development => 1,
    };

    let otp = match existing_otp {
        Some(existing_otp) => {
            repository::otp::update_by_id(
                ctx.db_conn.clone(),
                existing_otp.id.clone(),
                repository::otp::UpdateOtpPayload {
                    hash: Some(hash.clone()),
                    purpose: Some(purpose.clone()),
                    meta: Some(otp.pin_id.clone()),
                    validity,
                },
            )
            .await
        }
        None => {
            repository::otp::create(
                ctx.db_conn.clone(),
                repository::otp::CreateOtpPayload {
                    purpose,
                    meta: otp.pin_id.clone(),
                    hash,
                    otp: otp.pin_id.clone(),
                    validity,
                },
            )
            .await
        }
    }
    .map_err(|_| SendError::NotSent)?;

    Ok(otp)
}

pub async fn verify(
    ctx: Arc<Context>,
    user: repository::user::User,
    purpose: String,
    code: String,
) -> Result<(), VerificationError> {
    let hash = generate_hash(&purpose, &user);

    let existing_otp = repository::otp::find_by_hash(ctx.db_conn.clone(), hash.clone())
        .await
        .map_err(|_| VerificationError::UnexpectedError)?
        .ok_or(VerificationError::UnexpectedError)?;

    if Utc::now().naive_utc() > existing_otp.expires_at {
        return Err(VerificationError::Expired);
    }

    let res = hit_up_endpoint_and_parse(
        ctx.otp.verify_endpoint.clone(),
        json!({
            "api_key":ctx.otp.api_key.clone(),
            "pin_id":existing_otp.otp.clone(),
            "pin" : code.clone()
        })
        .to_string(),
    )
    .await
    .map_err(|_| VerificationError::UnexpectedError)?;

    if res.verified == "true" || res.pin_id != existing_otp.otp {
        return Err(VerificationError::InvalidOtp);
    }

    repository::otp::delete_by_id(ctx.db_conn.clone(), existing_otp.id).await;

    Ok(())
}
