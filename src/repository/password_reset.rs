use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use ulid::Ulid;

use crate::repository;

use crate::utils::database::DatabaseConnection;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PasswordReset {
    pub id: String,
    pub code: String,
    pub user_id: String,
    pub hash_proof: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreatePasswordResetPayload {
    pub code: String,
    pub user_id: String,
    pub hash_proof: String,
    pub expires_at: NaiveDateTime,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(
    db: DatabaseConnection,
    payload: CreatePasswordResetPayload,
) -> Result<PasswordReset, Error> {
    match sqlx::query_as!(
        PasswordReset,
        "
        INSERT INTO password_reset (
            id, 
            code, 
            hash_proof,
            expires_at,
            user_id
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        ",
        Ulid::new().to_string(),
        payload.code,
        payload.hash_proof,
        payload.expires_at,
        payload.user_id,
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(meal) => Ok(meal),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to create a password reset: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(
    db: DatabaseConnection,
    id: String,
) -> Result<Option<PasswordReset>, Error> {
    match sqlx::query_as!(
        PasswordReset,
        "SELECT * FROM password_reset WHERE id = $1",
        id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_pr) => Ok(maybe_pr),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch a password reset by id: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_hash_proof(
    db: DatabaseConnection,
    hash_proof: String,
) -> Result<Option<PasswordReset>, Error> {
    match sqlx::query_as!(
        PasswordReset,
        "SELECT * FROM password_reset WHERE hash_proof = $1",
        hash_proof
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_pr) => Ok(maybe_pr),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch a password reset by hash_proof: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<PasswordReset>,
    total: u32,
}

impl Into<DatabaseCountedResult> for Option<serde_json::Value> {
    fn into(self) -> DatabaseCountedResult {
        match self {
            Some(json) => {
                match serde_json::de::from_str::<DatabaseCountedResult>(json.to_string().as_ref()) {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!("{}", err);
                        DatabaseCountedResult {
                            data: vec![],
                            total: 0,
                        }
                    }
                }
            }
            None => DatabaseCountedResult {
                data: vec![],
                total: 0,
            },
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCounted {
    result: DatabaseCountedResult,
}

#[derive(Serialize)]
pub struct UpdatePasswordResetPayload {
    pub code: String,
    pub hash_proof: String,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdatePasswordResetPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE password_reset SET
                code = $1,
                hash_proof = $2,
                updated_at = NOW()
            WHERE
                id = $3
        ",
        payload.code,
        payload.hash_proof,
        id,
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            tracing::error!(
                "Error occurred while trying to update a password reset by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}

pub async fn delete_by_id(db: DatabaseConnection, id: String) -> Result<(), Error> {
    match sqlx::query_as!(
        PasswordReset,
        "DELETE FROM password_reset WHERE id = $1 RETURNING *",
        id
    )
    .fetch_one(&db.pool)
    .await
    {
        Err(e) => {
            tracing::error!(
                "Error occurred while trying to delete a password reset by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        Ok(_) => Ok(()),
    }
}
