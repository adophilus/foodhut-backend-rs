use std::ops::Deref;

use chrono::{NaiveDate, NaiveDateTime};
use log;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{self, database::DatabaseConnection};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfilePicture(pub Option<utils::storage::UploadedMedia>);

impl From<Option<serde_json::Value>> for ProfilePicture {
    fn from(value: Option<serde_json::Value>) -> Self {
        match value {
            Some(value) => serde_json::de::from_str::<Self>(value.to_string().as_str())
                .expect("Invalid user profile_picture found"),
            None => ProfilePicture(None),
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub phone_number: String,
    pub is_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub has_kitchen: bool,
    pub birthday: NaiveDateTime,
    pub referral_code: Option<String>,
    pub profile_picture: ProfilePicture,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateUserPayload {
    pub email: String,
    pub phone_number: String,
    pub first_name: String,
    pub last_name: String,
    pub birthday: NaiveDate,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateUserPayload) -> Result<(), Error> {
    let res = sqlx::query!(
        "INSERT INTO users (id, email, phone_number, first_name, last_name, birthday, is_verified) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        Ulid::new().to_string(),
        payload.email,
        payload.phone_number,
        payload.first_name,
        payload.last_name,
        payload.birthday.into(),
        false
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

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Option<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(&db.pool)
        .await
        .map_err(|err| {
            log::error!("Error occurred while fetching user with id {}: {}", id, err);
        })
        .unwrap_or(None)
}

pub async fn find_by_email(db: DatabaseConnection, email: String) -> Option<User> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(&db.pool)
        .await;

    match user {
        Ok(maybe_user) => maybe_user,
        Err(e) => {
            log::error!("Error occurred in find_by_email: {}", e);
            None
        }
    }
}

pub async fn find_by_phone_number(db: DatabaseConnection, phone_number: String) -> Option<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE phone_number = $1",
        phone_number
    )
    .fetch_optional(&db.pool)
    .await;

    match user {
        Ok(maybe_user) => maybe_user,
        Err(e) => {
            log::error!("Error occurred in find_by_phone_number: {}", e);
            None
        }
    }
}

pub async fn verify_by_phone_number(
    db: DatabaseConnection,
    phone_number: String,
) -> Result<(), Error> {
    match sqlx::query!(
        "UPDATE users SET is_verified = true WHERE phone_number = $1",
        phone_number
    )
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
        _ => Ok(()),
    }
}

pub struct UpdateUserPayload {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub has_kitchen: Option<bool>,
    pub profile_picture: Option<utils::storage::UploadedMedia>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateUserPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE users SET
                email = COALESCE($1, email),
                phone_number = COALESCE($2, phone_number),
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                birthday = COALESCE($5, birthday),
                has_kitchen = COALESCE($6, has_kitchen),
                profile_picture = COALESCE(
                    CASE WHEN $7::text = 'null' THEN NULL ELSE $7::json END, 
                    profile_picture
                ),
                updated_at = NOW()
            WHERE
                id = $8
        ",
        payload.email,
        payload.phone_number,
        payload.first_name,
        payload.last_name,
        payload.birthday,
        payload.has_kitchen,
        json!(payload.profile_picture).to_string(),
        id,
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!(
                "Error occurred while trying to update a user by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}
