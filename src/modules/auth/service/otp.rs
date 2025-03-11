use chrono::Utc;

use hyper::HeaderMap;
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};
use sha2::Digest;
use sqlx::{PgExecutor, Postgres, Transaction};

use crate::{
    modules::{auth::repository, notification, user, user::repository::User},
    types::{AppEnvironment, Context},
};
use std::sync::Arc;

#[derive(Eq, PartialEq, Debug)]
pub enum SendError {
    NotSent,
    NotExpired,
}

pub enum VerificationError {
    Expired,
    InvalidOtp,
    UnexpectedError,
}

// Putting this here because for some reason, the OTP service returns a boolean if everything works out well but then it returns a string if it doesn't
enum VerifiedEndpointStatus {
    Successful,
    Error(String),
}

impl<'de> Deserialize<'de> for VerifiedEndpointStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Bool(true) => Ok(VerifiedEndpointStatus::Successful),
            Value::Bool(false) => Ok(VerifiedEndpointStatus::Error("Invalid OTP".to_string())),
            Value::String(s) => Ok(VerifiedEndpointStatus::Error(s)),
            _ => Err(serde::de::Error::custom("Invalid verified endpoint status")),
        }
    }
}

#[derive(Deserialize)]
struct VerificationEndpointPayload {
    verified: VerifiedEndpointStatus,
    #[serde(rename = "pinId")]
    pin_id: String,
}

fn generate_hash(purpose: &str, user: &User) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{}-{}", purpose, user.id.clone()));
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

    tracing::info!("text: {}", text);

    serde_json::from_str::<_>(&text).map_err(|err| {
        tracing::error!("Failed to deserialize text: {}", err);
        SendError::NotSent
    })
}

pub async fn create(
    ctx: Arc<Context>,
    user: User,
    purpose: String,
    otp: String,
) -> Result<repository::otp::Otp, SendError> {
    let hash = generate_hash(&purpose, &user);

    let existing_otp = repository::otp::find_by_hash(&ctx.db_conn.pool, hash.clone())
        .await
        .map_err(|_| SendError::NotSent)?;

    if existing_otp.is_some() && Utc::now().naive_utc() <= existing_otp.clone().unwrap().expires_at
    {
        return Err(SendError::NotExpired);
    }

    let validity = match ctx.app.environment {
        AppEnvironment::Production => 3,
        AppEnvironment::Development => 1,
    };

    let otp = match existing_otp {
        Some(existing_otp) => {
            repository::otp::update_by_id(
                &ctx.db_conn.pool,
                existing_otp.id.clone(),
                repository::otp::UpdateOtpPayload {
                    hash: hash.clone(),
                    otp,
                    purpose: purpose.clone(),
                    meta: Some("".to_string()),
                    validity,
                },
            )
            .await
        }
        None => {
            repository::otp::create(
                &ctx.db_conn.pool,
                repository::otp::CreateOtpPayload {
                    purpose,
                    meta: otp.clone(),
                    hash,
                    otp: otp.clone(),
                    validity,
                },
            )
            .await
        }
    }
    .map_err(|_| SendError::NotSent)?;

    Ok(otp)
}

pub async fn send(
    ctx: Arc<Context>,
    user: User,
    purpose: String,
) -> Result<repository::otp::Otp, SendError> {
    let hash = generate_hash(&purpose, &user);

    let existing_otp = repository::otp::find_by_hash(&ctx.db_conn.pool, hash.clone())
        .await
        .map_err(|_| SendError::NotSent)?;

    if existing_otp.is_some() && Utc::now().naive_utc() <= existing_otp.clone().unwrap().expires_at
    {
        return Err(SendError::NotExpired);
    }

    let otp = notification::service::sms::send(
        ctx.clone(),
        notification::service::Notification::verification_otp_requested(user.clone()),
    )
    .await
    .map_err(|_| SendError::NotSent)?;

    let validity = match ctx.app.environment {
        AppEnvironment::Production => 3,
        AppEnvironment::Development => 1,
    };

    let otp = match existing_otp {
        Some(existing_otp) => {
            repository::otp::update_by_id(
                &ctx.db_conn.pool,
                existing_otp.id.clone(),
                repository::otp::UpdateOtpPayload {
                    hash: hash.clone(),
                    otp: otp.pin_id.clone(),
                    purpose: purpose.clone(),
                    meta: Some("".to_string()),
                    validity,
                },
            )
            .await
        }
        None => {
            repository::otp::create(
                &ctx.db_conn.pool,
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
    tx: &mut Transaction<'_, Postgres>,
    user: User,
    purpose: String,
    code: String,
) -> Result<(), VerificationError> {
    let hash = generate_hash(&purpose, &user);

    let existing_otp = repository::otp::find_by_hash(&mut **tx, hash.clone())
        .await
        .map_err(|_| VerificationError::UnexpectedError)?
        .ok_or(VerificationError::UnexpectedError)?;

    tracing::info!("found otp by hash! {:?}", existing_otp.clone());

    if Utc::now().naive_utc() > existing_otp.expires_at {
        tracing::info!("otp has expired");
        tracing::info!("otp expires at: {}", existing_otp.expires_at);
        tracing::info!("current time: {}", Utc::now().naive_utc());
        return Err(VerificationError::Expired);
    }

    match user::repository::find_exempt_by_id(&mut **tx, user.id.clone()).await {
        Ok(Some(_)) => {}
        Ok(_) => {
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

            if let VerifiedEndpointStatus::Error(err) = res.verified {
                tracing::debug!("Got an error response when verifying OTP: {}", err);
                return Err(VerificationError::InvalidOtp);
            }

            if res.pin_id != existing_otp.otp {
                return Err(VerificationError::InvalidOtp);
            }
        }
        _ => return Err(VerificationError::UnexpectedError),
    };

    repository::otp::delete_by_id(&mut **tx, existing_otp.id)
        .await
        .ok();

    Ok(())
}
