use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use std::convert::Into;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PushToken {
    pub id: String,
    pub token: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreatePushTokenPayload {
    pub token: String,
    pub user_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreatePushTokenPayload,
) -> Result<PushToken, Error> {
    match sqlx::query_as!(
        PushToken,
        "
        INSERT INTO push_tokens (
            id, 
            token,
            user_id 
        )
        VALUES ($1, $2, $3)
        RETURNING *
        ",
        Ulid::new().to_string(),
        payload.token,
        payload.user_id,
    )
    .fetch_one(e)
    .await
    {
        Ok(push_token) => Ok(push_token),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to create a push token: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}
