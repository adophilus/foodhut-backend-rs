use chrono::{DateTime, NaiveDateTime, Utc};
use log::debug;
use ulid::Ulid;

use crate::utils::database::DatabaseConnection;

pub struct Otp {
    pub id: String,
    pub otp: String,
    pub purpose: String,
    pub meta: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug)]
pub enum Error {
    OtpExpired,
    OtpNotFound,
    UnexpectedError,
    OtpCreationFailed,
    OtpNotExpired,
    InvalidOtp,
}

pub async fn create(
    db: DatabaseConnection,
    purpose: String,
    meta: String,
) -> Result<String, Error> {
    match find_one(db.clone(), purpose.clone(), meta.clone()).await {
        Err(Error::OtpExpired) => {
            match sqlx::query!(
                "DELETE FROM otps WHERE purpose = $1 AND meta = $2",
                purpose,
                meta
            )
            .execute(&db.pool)
            .await
            {
                Err(e) => {
                    log::error!("Error occurred while cleaning up expired OTP: {}", e);
                }
                _ => (),
            }

            create_otp(db, purpose, meta).await
        }
        Err(Error::OtpNotFound) => create_otp(db, purpose, meta).await,
        _ => Err(Error::OtpNotExpired),
    }
}

async fn create_otp(
    db: DatabaseConnection,
    purpose: String,
    meta: String,
) -> Result<String, Error> {
    let otp = "1234";

    match sqlx::query!(
        "INSERT INTO otps (id, purpose, meta, otp, expires_at) VALUES ($1, $2, $3, $4, $5)",
        Ulid::new().to_string(),
        purpose,
        meta,
        otp,
        // Utc::now().naive_utc() + chrono::Duration::minutes(5)
        Utc::now().naive_utc() + chrono::Duration::minutes(1)
    )
    .execute(&db.pool)
    .await
    {
        Ok(_) => Ok(otp.to_string()),
        Err(e) => {
            log::error!("{}", e);
            Err(Error::OtpCreationFailed)
        }
    }
}

pub async fn find_one(db: DatabaseConnection, purpose: String, meta: String) -> Result<Otp, Error> {
    let res = sqlx::query_as!(
        Otp,
        "SELECT * FROM otps WHERE purpose = $1 AND meta = $2",
        purpose,
        meta
    )
    .fetch_optional(&db.pool)
    .await;

    match res {
        Ok(Some(otp)) => {
            if Utc::now().naive_utc() > otp.expires_at {
                Err(Error::OtpExpired)
            } else {
                Ok(otp)
            }
        }
        Ok(None) => Err(Error::OtpNotFound),
        Err(e) => {
            log::error!("{}", e);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn verify(
    db: DatabaseConnection,
    purpose: String,
    meta: String,
    otp: String,
) -> Result<Otp, Error> {
    match find_one(db.clone(), purpose.clone(), meta.clone()).await {
        Ok(db_otp) => {
            if db_otp.otp != otp {
                return Err(Error::InvalidOtp);
            }

            match sqlx::query!("DELETE FROM otps WHERE id = $1", db_otp.id)
                .execute(&db.pool)
                .await
            {
                Err(e) => {
                    log::error!(
                        "Error occurred while trying to clean up verified OTP: {}",
                        e
                    );
                    return Err(Error::UnexpectedError);
                }
                _ => (),
            }

            Ok(db_otp)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(Error::UnexpectedError)
        }
    }
}
