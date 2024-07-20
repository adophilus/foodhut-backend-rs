use crate::types::StorageContext;
use reqwest::{
    multipart::{Form, Part},
    Body, Client, StatusCode, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{path::Path, time::UNIX_EPOCH};
use ulid::Ulid;

#[derive(Debug)]
pub enum Error {
    UploadFailed,
    DeleteFailed,
}

#[derive(Deserialize)]
struct UploadResponse {
    secure_url: String,
    signature: String,
    public_id: String,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct UploadedMedia {
    pub public_id: String,
    pub url: String,
    pub timestamp: i64,
}

impl From<Value> for UploadedMedia {
    fn from(value: Value) -> Self {
        match serde_json::de::from_str::<Self>(value.to_string().as_ref()) {
            Ok(media) => media,
            Err(_) => UploadedMedia {
                public_id: String::from(""),
                url: String::from(""),
                timestamp: 0,
            },
        }
    }
}

pub async fn upload_file(cfg: StorageContext, contents: Vec<u8>) -> Result<UploadedMedia, Error> {
    let file_name = Ulid::new().to_string();
    let part = Part::bytes(contents).file_name(file_name.clone());

    let timestamp = chrono::Utc::now().timestamp();
    let data_to_sign = format!(
        "timestamp={}&upload_preset={}{}",
        timestamp, cfg.upload_preset, cfg.api_secret
    );

    let mut hasher = Sha256::new();
    hasher.update(data_to_sign.clone());
    let hash = hasher.finalize();
    let signature = base16ct::lower::encode_string(&hash);

    let form = Form::new()
        .text("upload_preset", cfg.upload_preset.clone())
        .text("api_key", cfg.api_key.clone())
        .text("timestamp", format!("{}", timestamp))
        .text("signature", signature)
        .text("signature_algorithm", "sha256")
        .part("file", part);

    let res = Client::new()
        .post(cfg.upload_endpoint)
        .multipart(form)
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Error occurred while trying to upload a file: {:?}", err);
            Error::UploadFailed
        })?;

    if res.status() != StatusCode::OK {
        let data = res.text().await.map_err(|err| {
            tracing::error!("Error occurred while processing return data: {:?}", err);
            Error::UploadFailed
        })?;

        tracing::error!("Failed to upload file: {}", data);
        return Err(Error::UploadFailed);
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!("Error occurred while processing return data: {:?}", err);
        Error::UploadFailed
    })?;

    match serde_json::de::from_str::<UploadResponse>(data.as_ref()) {
        Ok(res) => Ok(UploadedMedia {
            url: res.secure_url,
            public_id: res.public_id,
            timestamp,
        }),
        Err(err) => {
            tracing::error!("Failed to deserialize cloudinary response: {:?}", err);
            Err(Error::UploadFailed)
        }
    }
}

pub async fn delete_file(cfg: StorageContext, media: UploadedMedia) -> Result<(), Error> {
    let url = Url::parse(media.url.as_ref()).map_err(|err| {
        tracing::error!("Failed to parse url {}: {:?}", media.url, err);
        Error::DeleteFailed
    })?;

    let data_to_sign = format!(
        "public_id={}&timestamp={}{}",
        media.public_id, media.timestamp, cfg.api_secret
    );

    let mut hasher = Sha256::new();
    hasher.update(data_to_sign.clone());
    let hash = hasher.finalize();
    let signature = base16ct::lower::encode_string(&hash);

    let body = json!({
        "public_id": media.public_id,
        "api_key": cfg.api_key,
        "signature": signature,
        "timestamp": media.timestamp,
    })
    .to_string();

    tracing::debug!("{}", body.clone());

    let res = Client::new()
        .post(cfg.delete_endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Failed to delete file {}: {:?}", url, err);
            Error::DeleteFailed
        })?;

    if res.status() != StatusCode::OK {
        let data = res.text().await.map_err(|err| {
            tracing::error!("Failed to process delete file response {}: {:?}", url, err);
            Error::DeleteFailed
        })?;

        tracing::error!("Failed to delete uploaded file: {}", data);
        return Err(Error::UploadFailed);
    }

    let data = res.text().await.map_err(|err| {
        tracing::error!("Failed to process delete file response {}: {:?}", url, err);
        Error::DeleteFailed
    })?;

    tracing::debug!("Delete file response: {}", data);

    Ok(())
}
