use crate::utils;
use crate::utils::storage::UploadedMedia;
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use std::convert::Into;
use ulid::Ulid;

use crate::repository;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rating: BigDecimal,
    pub price: BigDecimal,
    pub likes: i32,
    pub cover_image: utils::storage::UploadedMedia,
    pub is_available: bool,
    pub kitchen_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MealUserReaction {
    pub id: String,
    pub reaction: String,
    pub user_id: String,
    pub meal_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub enum MealUserReactionReaction {
    Like,
}

impl ToString for MealUserReactionReaction {
    fn to_string(&self) -> String {
        match self {
            Self::Like => String::from("LIKE"),
        }
    }
}

pub struct CreateMealPayload {
    pub name: String,
    pub description: String,
    pub price: BigDecimal,
    pub cover_image: utils::storage::UploadedMedia,
    pub kitchen_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateMealPayload) -> Result<Meal, Error> {
    match sqlx::query_as!(
        Meal,
        "
        INSERT INTO meals (
            id, 
            name, 
            description, 
            price,
            rating, 
            likes,
            cover_image, 
            is_available, 
            kitchen_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        ",
        Ulid::new().to_string(),
        payload.name,
        payload.description,
        payload.price,
        BigDecimal::from_u8(0).unwrap(),
        0,
        json!(payload.cover_image),
        true,
        payload.kitchen_id,
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(meal) => Ok(meal),
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
                tracing::debug!("{}", json);
                tracing::debug!("{}", json["data"][0]);
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
    pub cover_image: Option<UploadedMedia>,
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
                cover_image = COALESCE($5, cover_image),
                is_available = COALESCE($6, is_available),
                kitchen_id = COALESCE($7, kitchen_id),
                updated_at = NOW()
            WHERE
                id = $8
        ",
        payload.name,
        payload.description,
        payload.rating,
        payload.price,
        json!(payload.cover_image),
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
    match sqlx::query_as!(Meal, "DELETE FROM meals WHERE id = $1 RETURNING *", id)
        .fetch_one(&db.pool)
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

pub async fn like_by_id(db: DatabaseConnection, id: String, user_id: String) -> Result<(), Error> {
    match db.pool.begin().await {
        Ok(mut tx) => {
            match sqlx::query!(
                "SELECT * FROM meal_user_reactions WHERE meal_id = $1 AND user_id = $2",
                id.clone(),
                user_id
            )
            .fetch_one(&mut *tx)
            .await
            {
                Ok(_) => return Ok(()),
                Err(_) => (),
            }

            let reaction_id = Ulid::new().to_string();

            let insert_result = sqlx::query!(
                "
                    INSERT INTO meal_user_reactions (id, reaction, user_id, meal_id)
                    VALUES ($1, $2, $3, $4);
                ",
                reaction_id.clone(),
                MealUserReactionReaction::Like.to_string(),
                user_id,
                id.clone()
            )
            .execute(&mut *tx)
            .await;

            let update_result = sqlx::query!(
                "
                    UPDATE meals SET
                        likes = likes + 1,
                        updated_at = NOW()
                    WHERE
                        id = $1;
                ",
                id.clone()
            )
            .execute(&mut *tx)
            .await;

            match (insert_result, update_result) {
                (Ok(_), Ok(_)) => {
                    if let Err(e) = tx.commit().await {
                        tracing::error!("Failed to commit transaction: {}", e);
                        return Err(Error::UnexpectedError);
                    }
                    Ok(())
                }
                _ => {
                    if let Err(e) = tx.rollback().await {
                        tracing::error!("Failed to rollback transaction: {}", e);
                    }
                    Err(Error::UnexpectedError)
                }
            }
        }
        Err(err) => {
            tracing::error!("Failed to begin transaction: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn unlike_by_id(
    db: DatabaseConnection,
    id: String,
    user_id: String,
) -> Result<(), Error> {
    match db.pool.begin().await {
        Ok(mut tx) => {
            match sqlx::query!(
                "SELECT * FROM meal_user_reactions WHERE meal_id = $1 AND user_id = $2",
                id.clone(),
                user_id
            )
            .fetch_one(&mut *tx)
            .await
            {
                Ok(_) => (),
                Err(_) => return Ok(()),
            }

            tracing::info!("Got past the query for user_id and meal_id");

            let insert_result = sqlx::query!(
                "
                    DELETE FROM meal_user_reactions
                    WHERE meal_id = $1 AND user_id = $2
                ",
                id.clone(),
                user_id,
            )
            .execute(&mut *tx)
            .await;

            let update_result = sqlx::query!(
                "
                    UPDATE meals SET
                        likes = likes - 1,
                        updated_at = NOW()
                    WHERE
                        id = $1;
                ",
                id.clone()
            )
            .execute(&mut *tx)
            .await;

            match (insert_result, update_result) {
                (Ok(_), Ok(_)) => {
                    if let Err(e) = tx.commit().await {
                        tracing::error!("Failed to commit transaction: {}", e);
                        return Err(Error::UnexpectedError);
                    }
                    Ok(())
                }
                _ => {
                    if let Err(e) = tx.rollback().await {
                        tracing::error!("Failed to rollback transaction: {}", e);
                    }
                    Err(Error::UnexpectedError)
                }
            }
        }
        Err(err) => {
            tracing::error!("Failed to begin transaction: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub fn is_owner(
    user: repository::user::User,
    kitchen: repository::kitchen::Kitchen,
    meal: Meal,
) -> bool {
    return repository::kitchen::is_owner(user, kitchen.clone()) || kitchen.id == meal.kitchen_id;
}
