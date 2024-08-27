use std::ops::Deref;

use chrono::{NaiveDate, NaiveDateTime};
use log;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{self, database::DatabaseConnection};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone)]
pub struct BannerImage(pub Option<utils::storage::UploadedMedia>);

impl From<Option<serde_json::Value>> for BannerImage {
    fn from(value: Option<serde_json::Value>) -> Self {
        match value {
            Some(value) => serde_json::de::from_str::<Self>(value.to_string().as_str())
                .expect("Invalid ad banner_image found"),
            None => BannerImage(None),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ad {
    pub id: String,
    pub banner_image: BannerImage,
    pub link: String,
    pub duration: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateAdPayload {
    pub banner_image: String,
    pub link: String,
    pub duration: i32,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateAdPayload) -> Result<(), Error> {
    let res = sqlx::query!(
        "INSERT INTO ads (id, banner_image, link, duration) VALUES ($1, $2, $3, $4)",
        Ulid::new().to_string(),
        json!(payload.banner_image),
        payload.link,
        payload.duration,
    )
    .execute(&db.pool)
    .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{}", e);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Option<Ad> {
    sqlx::query_as!(Ad, "SELECT * FROM ads WHERE id = $1", id)
        .fetch_optional(&db.pool)
        .await
        .map_err(|err| {
            log::error!("Error occurred while fetching ad with id {}: {}", id, err);
        })
        .unwrap_or(None)
}

pub struct UpdateAdPayload {
    pub banner_image: Option<utils::storage::UploadedMedia>,
    pub link: Option<String>,
    pub duration: Option<i32>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateAdPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE ads SET
                banner_image = COALESCE(
                    CASE WHEN $1::text = 'null' THEN NULL ELSE $1::json END,
                    banner_image
                ),
                link = COALESCE($2, link),
                duration = COALESCE($3, duration),
                updated_at = NOW()
            WHERE
                id = $4
        ",
        payload.banner_image,
        payload.link,
        payload.duration,
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!(
                "Error occurred while trying to update a ad by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}
