use crate::define_paginated;
use crate::modules::storage;
use crate::utils::pagination::{Paginated, Pagination};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgExecutor;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ad {
    pub id: String,
    pub banner_image: storage::UploadedMedia,
    pub link: String,
    pub duration: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

define_paginated!(DatabasePaginatedAd, Ad);

pub struct CreateAdPayload {
    pub banner_image: storage::UploadedMedia,
    pub link: String,
    pub duration: i32,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E: PgExecutor<'e>>(e: E, payload: CreateAdPayload) -> Result<Ad, Error> {
    match sqlx::query_as!(
        Ad,
        "
        INSERT INTO ads
        (id, banner_image, link, duration)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        ",
        Ulid::new().to_string(),
        json!(payload.banner_image),
        payload.link,
        payload.duration,
    )
    .fetch_one(e)
    .await
    {
        Ok(ad) => Ok(ad),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Option<Ad>, Error> {
    match sqlx::query_as!(Ad, "SELECT * FROM ads WHERE id = $1", id)
        .fetch_optional(e)
        .await
    {
        Ok(ad) => Ok(ad),
        Err(err) => {
            tracing::error!("Error occurred while fetching ad with id {}: {}", id, err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
pub struct Filters {
    search: Option<String>,
}

pub async fn find_many<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: Filters,
) -> Result<Paginated<Ad>, Error> {
    sqlx::query_as!(
        DatabasePaginatedAd,
        "
            WITH filtered_data AS (
                SELECT *
                FROM ads 
                WHERE
                    link ILIKE CONCAT('%', COALESCE($3, link), '%')
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM ads 
                WHERE
                    link ILIKE CONCAT('%', COALESCE($3, link), '%')
            )
            SELECT 
                COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb) as items,
                JSONB_BUILD_OBJECT(
                    'total', (SELECT total_rows FROM total_count),
                    'per_page', $1,
                    'page', $2 / $1 + 1
                ) AS meta
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
        filters.search,
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedAd::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many ads: {}", err);
        Error::UnexpectedError
    })
}

pub struct UpdateAdPayload {
    pub banner_image: Option<storage::UploadedMedia>,
    pub link: Option<String>,
    pub duration: Option<i32>,
}

pub async fn update_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    payload: UpdateAdPayload,
) -> Result<(), Error> {
    sqlx::query!(
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
        json!(payload.banner_image).to_string(),
        payload.link,
        payload.duration,
        id,
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|e| {
        tracing::error!(
            "Error occurred while trying to update a ad by id {}: {}",
            id,
            e
        );
        Error::UnexpectedError
    })
}

pub async fn delete_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Ad, Error> {
    sqlx::query_as!(Ad, "DELETE FROM ads WHERE id = $1 RETURNING *", id)
        .fetch_one(e)
        .await
        .map_err(|err| {
            tracing::error!(
                "Error occurred while trying to delete an ad by id {}: {}",
                id,
                err
            );
            Error::UnexpectedError
        })
}

pub async fn delete_expired<'e, E: PgExecutor<'e>>(e: E) -> Result<(), Error> {
    sqlx::query_as!(
        Ad,
        "DELETE FROM ads WHERE created_at + (duration * INTERVAL '1 second') < NOW()"
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!("Error occurred while trying to delete expired ads: {}", err);
        Error::UnexpectedError
    })
}
