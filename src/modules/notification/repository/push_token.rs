use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use std::convert::Into;
use ulid::Ulid;

use crate::{modules::user::repository::User, utils::pagination::{Paginated, Pagination}};

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

pub async fn find_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
) -> Result<Option<PushToken>, Error> {
    match sqlx::query_as!(PushToken, "SELECT * FROM push_tokens WHERE id = $1", id)
        .fetch_optional(e)
        .await
    {
        Ok(maybe_push_token) => Ok(maybe_push_token),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch many push tokens: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<PushToken>,
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

#[derive(Deserialize)]
pub struct Filters {
    user_id: Option<String>,
}

pub async fn find_many<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: Filters,
) -> Result<Paginated<PushToken>, Error> {
    sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM push_tokens 
                WHERE
                    user_id = COALESCE($3, user_id)
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM push_tokens
                WHERE
                    user_id = COALESCE($3, user_id)
            )
            SELECT JSONB_BUILD_OBJECT(
                'data', COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb),
                'total', (SELECT total_rows FROM total_count)
            ) AS result
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
        filters.user_id,
    )
    .fetch_one(e)
    .await
    .map(|counted| {
        Paginated::new(
            counted.result.data,
            counted.result.total,
            pagination.page,
            pagination.per_page,
        )
    })
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch many push tokens: {}",
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Serialize)]
pub struct UpdatePushTokenPayload {
    pub token: Option<String>,
    pub user_id: Option<String>,
}

pub async fn update_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    payload: UpdatePushTokenPayload,
) -> Result<(), Error> {
    sqlx::query!(
        "
        UPDATE push_tokens SET
            token = COALESCE($1, token),
            user_id = COALESCE($2, user_id),
            updated_at = NOW()
        WHERE
            id = $3
        ",
        payload.token,
        payload.user_id,
        id,
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to update a push token by id {}: {}",
            id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn delete_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<(), Error> {
    match sqlx::query_as!(
        PushToken,
        "DELETE FROM push_tokens WHERE id = $1 RETURNING *",
        id
    )
    .fetch_one(e)
    .await
    {
        Err(e) => {
            tracing::error!(
                "Error occurred while trying to delete a push token by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        Ok(_) => Ok(()),
    }
}

pub fn is_owner(user: &User, push_token: &PushToken) -> bool {
    push_token.user_id == user.id
}
