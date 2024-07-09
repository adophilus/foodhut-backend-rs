use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::convert::Into;
use ulid::Ulid;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Kitchen {
    pub id: String,
    pub name: String,
    pub address: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub phone_number: String,
    pub opening_time: String,
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
    pub cover_image_url: Option<String>,
    pub rating: BigDecimal,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateKitchenPayload {
    pub name: String,
    pub address: String,
    pub phone_number: String,
    pub type_: String,
    pub opening_time: String,
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
    pub owner_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateKitchenPayload) -> Result<(), Error> {
    match sqlx::query!(
        "
        INSERT INTO kitchens (
            id,
            name,
            address,
            type,
            phone_number,
            opening_time,
            closing_time,
            preparation_time,
            delivery_time,
            rating,
            owner_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    ",
        Ulid::new().to_string(),
        payload.name,
        payload.address,
        payload.type_,
        payload.phone_number,
        payload.opening_time,
        payload.closing_time,
        payload.preparation_time,
        payload.delivery_time,
        BigDecimal::new(BigInt::new(Sign::Plus, vec![0]), 2),
        payload.owner_id
    )
    .execute(&db.pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::info!("Error occurred while trying to create a kitchen: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Kitchen>, Error> {
    match sqlx::query_as!(
        Kitchen,
        "
            SELECT 
                id, 
                name, 
                address, 
                type AS type_, 
                phone_number, 
                opening_time, 
                closing_time, 
                preparation_time, 
                delivery_time, 
                cover_image_url, 
                rating, 
                owner_id, 
                created_at, 
                updated_at
            FROM kitchens WHERE id = $1
        ",
        id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_kitchen) => Ok(maybe_kitchen),
        Err(err) => {
            tracing::info!(
                "Error occurred while trying to fetch many kitchens: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_owner_id(
    db: DatabaseConnection,
    owner_id: String,
) -> Result<Option<Kitchen>, Error> {
    match sqlx::query_as!(
        Kitchen,
        "
            SELECT 
                id, 
                name, 
                address, 
                type AS type_, 
                phone_number, 
                opening_time, 
                closing_time, 
                preparation_time, 
                delivery_time, 
                cover_image_url, 
                rating, 
                owner_id, 
                created_at, 
                updated_at
            FROM kitchens WHERE owner_id = $1
        ",
        owner_id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_kitchen) => Ok(maybe_kitchen),
        Err(err) => {
            tracing::info!(
                "Error occurred while trying to fetch many kitchens: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Kitchen>,
    total: u32,
}

impl Into<DatabaseCountedResult> for Option<serde_json::Value> {
    fn into(self) -> DatabaseCountedResult {
        match self {
            Some(json) => {
                serde_json::de::from_str::<DatabaseCountedResult>(json.to_string().as_ref())
                    .unwrap()
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

pub async fn find_many(
    db: DatabaseConnection,
    pagination: Pagination,
) -> Result<Paginated<Kitchen>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM kitchens 
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM kitchens
            )
            SELECT JSONB_BUILD_OBJECT(
                'data', JSONB_AGG(ROW_TO_JSON(filtered_data)),
                'total', (SELECT total_rows FROM total_count)
            ) AS result
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
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
            tracing::info!(
                "Error occurred while trying to fetch many kitchens: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Serialize)]
pub struct UpdateKitchenPayload {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone_number: Option<String>,
    pub type_: Option<String>,
    pub opening_time: Option<String>,
    pub closing_time: Option<String>,
    pub preparation_time: Option<String>,
    pub delivery_time: Option<String>,
    pub rating: Option<BigDecimal>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateKitchenPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE kitchens SET
                name = COALESCE($1, name),
                address = COALESCE($2, address),
                type = COALESCE($3, type),
                phone_number = COALESCE($4, phone_number),
                opening_time = COALESCE($5, opening_time),
                closing_time = COALESCE($6, closing_time),
                preparation_time = COALESCE($7, preparation_time),
                delivery_time = COALESCE($8, delivery_time),
                rating = COALESCE($9, rating),
                updated_at = NOW()
            WHERE
                id = $10
        ",
        payload.name,
        payload.address,
        payload.type_,
        payload.phone_number,
        payload.opening_time,
        payload.closing_time,
        payload.preparation_time,
        payload.delivery_time,
        payload.rating,
        id,
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
