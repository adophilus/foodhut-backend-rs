use chrono::{DateTime, NaiveDateTime, Utc};
use log::debug;
use ulid::Ulid;

use crate::utils::database::DatabaseConnection;

#[derive(Clone)]
pub struct Otp {
    pub id: String,
    pub otp: String,
    pub purpose: String,
    pub meta: String,
    pub hash: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub struct CreateOtpPayload {
    pub purpose: String,
    pub meta: String,
    pub hash: String,
    pub otp: String,
}

pub async fn create(db: DatabaseConnection, payload: CreateOtpPayload) -> Result<Otp, Error> {
    sqlx::query_as!(
        Otp,
        "
        INSERT INTO otps (id, purpose, meta, otp, hash, expires_at) VALUES ($1, $2, $3, $4, $5, NOW() + INTERVAL '1 minute')
        RETURNING *
        ",
        Ulid::new().to_string(),
        payload.purpose,
        payload.meta,
        payload.otp,
        payload.hash
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
        log::error!("Error occurred while creating otp {}", e);
        Error::UnexpectedError
    })
}

pub async fn find_by_hash(db: DatabaseConnection, hash: String) -> Result<Option<Otp>, Error> {
    sqlx::query_as!(Otp, "SELECT * FROM otps WHERE hash = $1", hash)
        .fetch_optional(&db.pool)
        .await
        .map_err(|err| {
            tracing::error!("Error occurred while trying to fetch otp by hash: {}", err);
            Error::UnexpectedError
        })
}

pub struct UpdateOtpPayload {
    pub purpose: Option<String>,
    pub meta: Option<String>,
    pub hash: Option<String>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateOtpPayload,
) -> Result<Otp, Error> {
    sqlx::query_as!(
        Otp,
        "
            UPDATE otps SET
                purpose = COALESCE($1, purpose),
                meta = COALESCE($2, meta),
                hash = COALESCE($3, hash),
                expires_at = NOW() + INTERVAL '1 MINUTE',
                updated_at = NOW()
            WHERE
                id = $4
            RETURNING *
        ",
        payload.purpose,
        payload.meta,
        payload.hash,
        id
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to update otp by id {}: {}", id, err);
        Error::UnexpectedError
    })
}

pub async fn delete_by_id(db: DatabaseConnection, id: String) -> Result<(), Error> {
    sqlx::query!("DELETE FROM otps WHERE id = $1", id)
        .execute(&db.pool)
        .await
        .map_err(|err| {
            tracing::error!("Failed to delete otp by id {}: {}", id, err);
            Error::UnexpectedError
        })
        .map(|_| {})
}
