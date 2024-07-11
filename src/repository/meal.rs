use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use std::convert::Into;
use std::ops::{Deref, DerefMut};
use ulid::Ulid;

use crate::repository;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tags {
    pub items: Vec<String>,
}

impl Deref for Tags {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl Into<Tags> for serde_json::Value {
    fn into(self) -> Tags {
        tracing::debug!("Am I the one causing the error?");
        match self {
            serde_json::Value::Array(items) => Tags {
                items: items.into_iter().map(|item| item.to_string()).collect(),
            },
            _ => Tags { items: vec![] },
        };
        tracing::debug!("No I'm not");
        Tags { items: vec![] }
        // match serde_json::de::from_str::<Vec<String>>(self.to_string().as_ref()) {
        //     Ok(items) => Tags { items },
        //     Err(err) => {
        //         tracing::error!(
        //             "Error occurred while trying to convert tags from the db to Tags {}: {}",
        //             self.to_string(),
        //             err
        //         );
        //         Tags { items: vec![] }
        //     }
        // }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Meal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rating: BigDecimal,
    pub price: BigDecimal,
    pub likes: i32,
    pub tags: Tags,
    pub cover_image_url: String,
    pub is_available: bool,
    pub kitchen_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateMealPayload {
    pub name: String,
    pub description: String,
    pub price: BigDecimal,
    pub tags: Vec<String>,
    pub cover_image_url: String,
    pub kitchen_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateMealPayload) -> Result<(), Error> {
    match sqlx::query!(
        "
        INSERT INTO meals (
            id, 
            name, 
            description, 
            price,
            rating, 
            likes,
            tags, 
            cover_image_url, 
            is_available, 
            kitchen_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ",
        Ulid::new().to_string(),
        payload.name,
        payload.description,
        payload.price,
        BigDecimal::new(BigInt::new(Sign::Plus, vec![0]), 2),
        0,
        json!(payload.tags),
        payload.cover_image_url,
        true,
        payload.kitchen_id,
    )
    .execute(&db.pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::error!("Error occurred while trying to create a meal: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Meal>, Error> {
    match sqlx::query_as!(Meal, "SELECT * FROM meals WHERE id = $1", id)
        .fetch_optional(&db.pool)
        .await
    {
        Ok(maybe_meal) => Ok(maybe_meal),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch many meals: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Meal>,
    total: u32,
}

impl Into<DatabaseCountedResult> for Option<serde_json::Value> {
    fn into(self) -> DatabaseCountedResult {
        match self {
            Some(json) => {
                tracing::debug!("{}", json["data"]);
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

pub async fn find_many(
    db: DatabaseConnection,
    pagination: Pagination,
) -> Result<Paginated<Meal>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM meals 
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM meals
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
            tracing::error!("Error occurred while trying to fetch many meals: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Serialize)]
pub struct UpdateMealPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rating: Option<BigDecimal>,
    pub price: Option<BigDecimal>,
    pub tags: Option<Vec<String>>,
    pub cover_image_url: Option<String>,
    pub is_available: Option<bool>,
    pub kitchen_id: Option<String>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateMealPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE meals SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                rating = COALESCE($3, rating),
                price = COALESCE($4, price),
                tags = COALESCE($5, tags),
                cover_image_url = COALESCE($6, cover_image_url),
                is_available = COALESCE($7, is_available),
                kitchen_id = COALESCE($8, kitchen_id),
                updated_at = NOW()
            WHERE
                id = $9
        ",
        payload.name,
        payload.description,
        payload.rating,
        payload.price,
        payload.tags.map(|t| json!(t)),
        payload.cover_image_url,
        payload.is_available,
        payload.kitchen_id,
        id,
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!(
                "Error occurred while trying to update a meal by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}

pub async fn delete_by_id(db: DatabaseConnection, id: String) -> Result<(), Error> {
    match sqlx::query!("DELETE FROM meals WHERE id = $1", id)
        .execute(&db.pool)
        .await
    {
        Err(e) => {
            log::error!(
                "Error occurred while trying to delete a meal by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        Ok(_) => Ok(()),
    }
}

pub fn is_owner(
    user: repository::user::User,
    kitchen: repository::kitchen::Kitchen,
    meal: Meal,
) -> bool {
    return repository::kitchen::is_owner(user, kitchen.clone()) || kitchen.id == meal.kitchen_id;
}
