use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgExecutor;

use ulid::Ulid;

use crate::modules::storage::UploadedMedia;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfilePicture(pub Option<UploadedMedia>);

impl From<Option<serde_json::Value>> for ProfilePicture {
    fn from(value: Option<serde_json::Value>) -> Self {
        match value {
            Some(value) => serde_json::de::from_str::<Self>(value.to_string().as_str())
                .expect("Invalid user profile_picture found"),
            None => ProfilePicture(None),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    #[serde(rename = "ADMIN")]
    Admin,
    #[serde(rename = "USER")]
    User,
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "ADMIN" => Role::Admin,
            "USER" => Role::User,
            role => unreachable!("Invalid user role: {}", role),
        }
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Admin => String::from("ADMIN"),
            Role::User => String::from("USER"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub phone_number: String,
    pub is_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
    pub has_kitchen: bool,
    pub referral_code: Option<String>,
    pub profile_picture: ProfilePicture,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<sqlx::types::Json<User>> for User {
    fn from(value: sqlx::types::Json<User>) -> Self {
        value.0
    }
}

pub struct CreateUserPayload {
    pub email: String,
    pub phone_number: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E>(db: E, payload: CreateUserPayload) -> Result<User>
where
    E: PgExecutor<'e>,
{
    match sqlx::query_as!(
        User,
        "
        INSERT INTO users (id, email, phone_number, first_name, last_name, is_verified)
        VALUES ($1, $2, $3, $4, $5, false)
        RETURNING *
        ",
        Ulid::new().to_string(),
        payload.email,
        payload.phone_number,
        payload.first_name,
        payload.last_name,
    )
    .fetch_one(db)
    .await
    {
        Ok(user) => Ok(user),
        Err(err) => {
            tracing::error!("Error occured while creating a user account: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Option<User>> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(e)
        .await
        .map_err(|err| {
            tracing::error!("Error occurred while fetching user with id {}: {}", id, err);
            Error::UnexpectedError
        })
}

pub async fn find_by_kitchen_id<'e, E: PgExecutor<'e>>(
    e: E,
    kitchen_id: String,
) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        "
        SELECT
            users.*
        FROM
            users,
            kitchens
        WHERE
            kitchens.id = $1
            AND users.id = kitchens.owner_id
        ",
        kitchen_id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while fetching user with kitchen id {}: {}",
            kitchen_id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_by_email<'e, E: PgExecutor<'e>>(e: E, email: String) -> Result<Option<User>> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(e)
        .await
        .map_err(|err| {
            tracing::error!("Error occurred in find_by_email: {}", err);
            Error::UnexpectedError
        })
}

pub async fn find_by_phone_number<'e, E: PgExecutor<'e>>(
    e: E,
    phone_number: String,
) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE phone_number = $1",
        phone_number
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred in find_by_phone_number: {}", err);
        Error::UnexpectedError
    })
}

pub struct FindByEmailOrPhoneNumber {
    pub email: String,
    pub phone_number: String,
}

pub async fn find_by_email_or_phone_number<'e, E: PgExecutor<'e>>(
    e: E,
    payload: FindByEmailOrPhoneNumber,
) -> Result<Option<User>> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1 OR phone_number = $2",
        payload.email,
        payload.phone_number
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred in find_by_phone_number: {}", err);
        Error::UnexpectedError
    })
}

pub async fn verify_by_phone_number<'e, E: PgExecutor<'e>>(
    e: E,
    phone_number: String,
) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET is_verified = true WHERE phone_number = $1",
        phone_number
    )
    .execute(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to verify user by phone number: {}",
            err
        );
        Error::UnexpectedError
    })
    .map(|_| {})
}

pub struct UpdateUserPayload {
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub has_kitchen: Option<bool>,
    pub profile_picture: Option<UploadedMedia>,
}

pub async fn update_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    payload: UpdateUserPayload,
) -> Result<()> {
    sqlx::query!(
        "
            UPDATE users SET
                email = COALESCE($1, email),
                phone_number = COALESCE($2, phone_number),
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                has_kitchen = COALESCE($5, has_kitchen),
                profile_picture = COALESCE(
                    CASE WHEN $6::text = 'null' THEN NULL ELSE $6::json END, 
                    profile_picture
                ),
                updated_at = NOW()
            WHERE
                id = $7
        ",
        payload.email,
        payload.phone_number,
        payload.first_name,
        payload.last_name,
        payload.has_kitchen,
        json!(payload.profile_picture).to_string(),
        id,
    )
    .execute(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to update a user by id {}: {}",
            id,
            err
        );
        Error::UnexpectedError
    })
    .map(|_| ())
}

pub fn is_admin(user: &User) -> bool {
    return user.role == Role::Admin;
}
