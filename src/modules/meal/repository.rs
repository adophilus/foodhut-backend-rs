use crate::define_paginated;
use crate::modules::{
    kitchen::{self, repository::Kitchen},
    storage,
    user::repository::User,
};
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use sqlx::PgExecutor;
use std::convert::Into;
use ulid::Ulid;

use crate::utils::pagination::{Paginated, Pagination};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rating: BigDecimal,
    pub original_price: BigDecimal,
    pub price: BigDecimal,
    pub likes: i32,
    pub cover_image: storage::UploadedMedia,
    pub is_available: bool,
    pub kitchen_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MealWithCartStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rating: BigDecimal,
    pub original_price: BigDecimal,
    pub price: BigDecimal,
    pub likes: i32,
    pub cover_image: storage::UploadedMedia,
    pub is_available: bool,
    pub in_cart: bool,
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

define_paginated!(DatabasePaginatedMeal, Meal);

pub struct CreateMealPayload {
    pub name: String,
    pub description: String,
    pub price: BigDecimal,
    pub cover_image: storage::UploadedMedia,
    pub kitchen_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreateMealPayload,
) -> Result<Meal, Error> {
    match sqlx::query_as!(
        Meal,
        "
        INSERT INTO meals (
            id, 
            name, 
            description, 
            original_price,
            price,
            rating, 
            likes,
            cover_image, 
            is_available, 
            kitchen_id
        )
        VALUES ($1, $2, $3, $4, $4 + ($4 * 0.2), $5, $6, $7, $8, $9)
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
    .fetch_one(e)
    .await
    {
        Ok(meal) => Ok(meal),
        Err(err) => {
            tracing::error!("Error occurred while trying to create a meal: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Option<Meal>, Error> {
    match sqlx::query_as!(Meal, "SELECT * FROM meals WHERE id = $1", id)
        .fetch_optional(e)
        .await
    {
        Ok(maybe_meal) => Ok(maybe_meal),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch a meal by id: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
pub struct Filters {
    pub kitchen_id: Option<String>,
    pub search: Option<String>,
    pub is_liked_by: Option<String>,
}

pub async fn find_many<'e, E>(
    e: E,
    pagination: Pagination,
    filters: Filters,
) -> Result<Paginated<Meal>, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        DatabasePaginatedMeal,
        r#"
        WITH filtered_data AS (
            SELECT
                meals.*
            FROM
                meals
            LEFT JOIN
                meal_user_reactions 
            ON
                meals.id = meal_user_reactions.meal_id
            AND (
                $5::TEXT IS NOT NULL AND 
                meal_user_reactions.user_id = $5 AND 
                meal_user_reactions.reaction = 'LIKE'
            )
            WHERE
                meals.kitchen_id = COALESCE($3, meals.kitchen_id)
                AND meals.name ILIKE CONCAT('%', COALESCE($4, meals.name), '%')
                AND ($5::TEXT IS NULL OR meal_user_reactions.id IS NOT NULL)
            LIMIT $1
            OFFSET $2
        ),
        total_count AS (
            SELECT
                COUNT(meals.id) AS total_rows
            FROM
                meals
            LEFT JOIN meal_user_reactions 
            ON meals.id = meal_user_reactions.meal_id
            AND (
                $5::TEXT IS NOT NULL AND 
                meal_user_reactions.user_id = $5 AND 
                meal_user_reactions.reaction = 'LIKE'
            )
            WHERE
                meals.kitchen_id = COALESCE($3, meals.kitchen_id)
                AND meals.name ILIKE CONCAT('%', COALESCE($4, meals.name), '%')
                AND ($5::TEXT IS NULL OR meal_user_reactions.id IS NOT NULL)
        )
        SELECT 
            COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb) AS items,
            JSONB_BUILD_OBJECT(
                'total', (SELECT total_rows FROM total_count),
                'per_page', $1,
                'page', $2 / $1 + 1
            ) AS meta
        FROM filtered_data;
        "#,
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
        filters.kitchen_id,
        filters.search,
        filters.is_liked_by,
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedMeal::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many meals: {}", err);
        Error::UnexpectedError
    })
}

#[derive(Serialize)]
pub struct UpdateMealPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rating: Option<BigDecimal>,
    pub price: Option<BigDecimal>,
    pub cover_image: Option<storage::UploadedMedia>,
    pub is_available: Option<bool>,
    pub kitchen_id: Option<String>,
}

pub async fn update_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    payload: UpdateMealPayload,
) -> Result<(), Error> {
    sqlx::query!(
        "
            UPDATE meals SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                rating = COALESCE($3, rating),
                original_price = COALESCE($4, original_price),
                price = COALESCE($4, original_price) + (COALESCE($4, original_price) * 0.2),
                cover_image = COALESCE(
                    CASE WHEN $5::text = 'null' THEN NULL ELSE $5::json END, 
                    cover_image
                ),
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
        json!(payload.cover_image).to_string(),
        payload.is_available,
        payload.kitchen_id,
        id,
    )
    .execute(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to update a meal by id {}: {}",
            id,
            err
        );
        Error::UnexpectedError
    })
    .map(|_| ())
}

pub async fn delete_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<(), Error> {
    sqlx::query_as!(Meal, "DELETE FROM meals WHERE id = $1 RETURNING *", id)
        .fetch_one(e)
        .await
        .map(|_| ())
        .map_err(|err| {
            tracing::error!(
                "Error occurred while trying to delete a meal by id {}: {}",
                id,
                err
            );
            Error::UnexpectedError
        })
}

// TODO: cross check this function
pub async fn like_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    user_id: String,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        WITH insert_reaction AS (
            INSERT INTO meal_user_reactions (id, reaction, user_id, meal_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, meal_id) DO NOTHING
            RETURNING 1
        )
        UPDATE meals
        SET likes = likes + (SELECT COUNT(*) FROM insert_reaction),
            updated_at = NOW()
        WHERE id = $4;
        "#,
        Ulid::new().to_string(),
        MealUserReactionReaction::Like.to_string(),
        user_id,
        id
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!("Failed to execute like_by_id query: {}", err);
        Error::UnexpectedError
    })
}

// TODO: cross check this function
pub async fn unlike_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    user_id: String,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        WITH delete_reaction AS (
            DELETE FROM meal_user_reactions
            WHERE meal_id = $1 AND user_id = $2
            RETURNING 1
        )
        UPDATE meals
        SET likes = likes - (SELECT COUNT(*) FROM delete_reaction),
            updated_at = NOW()
        WHERE id = $1;
        "#,
        id,
        user_id
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!("Failed to execute unlike_by_id query: {}", err);
        Error::UnexpectedError
    })
}

pub fn is_owner(user: &User, kitchen: &Kitchen, meal: &Meal) -> bool {
    return kitchen::repository::is_owner(&user, &kitchen) || kitchen.id == meal.kitchen_id;
}
