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
    sqlx::query_as!(
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
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to create a push token: {}",
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_many_by_user_id<'e, E: PgExecutor<'e>>(
    e: E,
    user_id: String,
) -> Result<Vec<PushToken>, Error> {
    sqlx::query_as!(
        PushToken,
        "SELECT * FROM push_tokens WHERE user_id = $1",
        &user_id
    )
    .fetch_all(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to find many push tokens by user id {}: {:?}",
            user_id,
            err
        );
        Error::UnexpectedError
    })
}
