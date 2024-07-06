use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use ulid::Ulid;

use crate::{types::Pagination, utils::database::DatabaseConnection};

#[derive(Serialize, Deserialize)]
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
    pub rating: f64,
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

// pub async fn find_many(
//     db: DatabaseConnection,
//     pagination: Pagination,
// ) -> Result<Vec<Kitchen>, Error> {
//     match sqlx::query_as!(
//         Vec<Kitchen>,
//         "
//             SELECT * FROM kitchens
//             OFFSET $1
//             LIMIT $2
//         ",
//         ((pagination.page - 1) * pagination.per_page) as i64,
//         pagination.per_page as i64
//     )
//     .find_all(&db.pool)
//     .await
//     {
//         Ok(kitchens) => kitchens,
//         Err(err) {
//             tracing::info!("Error occurred while trying to fetch many kitchens: {}", err);
//             Err(Error::UnexpectedError)
//     }
//     }
// }
