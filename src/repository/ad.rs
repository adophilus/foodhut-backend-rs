use std::ops::Deref;

use chrono::{NaiveDate, NaiveDateTime};
use log;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{
    self,
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Ad {
    pub id: String,
    pub banner_image: utils::storage::UploadedMedia,
    pub link: String,
    pub duration: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateAdPayload {
    pub banner_image: utils::storage::UploadedMedia,
    pub link: String,
    pub duration: i32,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateAdPayload) -> Result<Ad, Error> {
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
    .fetch_one(&db.pool)
    .await
    {
        Ok(ad) => Ok(ad),
        Err(e) => {
            log::error!("{}", e);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Ad>, Error> {
    match sqlx::query_as!(Ad, "SELECT * FROM ads WHERE id = $1", id)
        .fetch_optional(&db.pool)
        .await
    {
        Ok(ad) => Ok(ad),
        Err(err) => {
            log::error!("Error occurred while fetching ad with id {}: {}", id, err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Ad>,
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
    search: Option<String>,
}

pub async fn find_many(
    db: DatabaseConnection,
    pagination: Pagination,
    filters: Filters,
) -> Result<Paginated<Ad>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
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
            SELECT JSONB_BUILD_OBJECT(
                'data', JSONB_AGG(ROW_TO_JSON(filtered_data)),
                'total', (SELECT total_rows FROM total_count)
            ) AS result
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
        filters.search,
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(counted) => Ok(Paginated::new(
            counted.result.data,
            counted.result.total,
            pagination.page,
            pagination.per_page,
        )),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch many ads: {}", err);
            Err(Error::UnexpectedError)
        }
    }
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
        json!(payload.banner_image).to_string(),
        payload.link,
        payload.duration,
        id,
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

pub async fn delete_by_id(db: DatabaseConnection, id: String) -> Result<(), Error> {
    match sqlx::query_as!(Ad, "DELETE FROM ads WHERE id = $1 RETURNING *", id)
        .fetch_one(&db.pool)
        .await
    {
        Err(err) => {
            log::error!(
                "Error occurred while trying to delete an ad by id {}: {}",
                id,
                err
            );
            return Err(Error::UnexpectedError);
        }
        Ok(_) => Ok(()),
    }
}
