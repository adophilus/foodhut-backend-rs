use crate::define_paginated;
use crate::modules::{
    cart::repository::Cart,
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
use std::convert::From;
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
    pub deleted_at: Option<NaiveDateTime>,
}

impl Meal {
    pub fn with_cart_status(self, cart: &Cart) -> MealWithCartStatus {
        MealWithCartStatus {
            id: self.id.clone(),
            name: self.name,
            description: self.description,
            rating: self.rating,
            original_price: self.original_price,
            price: self.price,
            likes: self.likes,
            cover_image: self.cover_image,
            is_available: self.is_available,
            in_cart: !cart
                .items
                .0
                .iter()
                .filter(|item| item.meal_id == self.id)
                .collect::<Vec<_>>()
                .is_empty(),
            kitchen_id: self.kitchen_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
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
    pub deleted_at: Option<NaiveDateTime>,
}

impl From<Meal> for MealWithCartStatus {
    fn from(meal: Meal) -> Self {
        Self {
            id: meal.id.clone(),
            name: meal.name,
            description: meal.description,
            rating: meal.rating,
            original_price: meal.original_price,
            price: meal.price,
            likes: meal.likes,
            cover_image: meal.cover_image,
            is_available: meal.is_available,
            in_cart: false,
            kitchen_id: meal.kitchen_id,
            created_at: meal.created_at,
            updated_at: meal.updated_at,
            deleted_at: meal.deleted_at,
        }
    }
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
pub struct FindManyAsAdminFilters {
    pub kitchen_id: Option<String>,
    pub search: Option<String>,
    pub is_liked_by: Option<String>,
}

pub async fn find_many_as_admin<'e, E>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsAdminFilters,
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
                AND deleted_at IS NULL
            LIMIT $2 OFFSET ($1 - 1) * $2
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
                AND deleted_at IS NULL
        )
        SELECT 
            COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM filtered_data;
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
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

#[derive(Deserialize)]
pub struct FindManyAsKitchenFilters {
    pub kitchen_id: Option<String>,
    pub search: Option<String>,
    pub is_liked_by: Option<String>,
    pub owner_id: String,
}

pub async fn find_many_as_kitchen<'e, E>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsKitchenFilters,
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
            INNER JOIN kitchens ON meals.kitchen_id = kitchens.id
            WHERE
                meals.kitchen_id = COALESCE($3, meals.kitchen_id)
                AND meals.name ILIKE CONCAT('%', COALESCE($4, meals.name), '%')
                AND ($5::TEXT IS NULL OR meal_user_reactions.id IS NOT NULL)
                AND (meals.is_available = TRUE OR kitchens.owner_id = $6)
                AND deleted_at IS NULL
            LIMIT $2 OFFSET ($1 - 1) * $2
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
            INNER JOIN kitchens ON meals.kitchen_id = kitchens.id
            WHERE
                meals.kitchen_id = COALESCE($3, meals.kitchen_id)
                AND meals.name ILIKE CONCAT('%', COALESCE($4, meals.name), '%')
                AND ($5::TEXT IS NULL OR meal_user_reactions.id IS NOT NULL)
                AND (meals.is_available = TRUE OR kitchens.owner_id = $6)
                AND deleted_at IS NULL
        )
        SELECT 
            COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM filtered_data;
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
        filters.kitchen_id,
        filters.search,
        filters.is_liked_by,
        filters.owner_id,
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedMeal::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many meals: {}", err);
        Error::UnexpectedError
    })
}

#[derive(Deserialize)]
pub struct FindManyAsUserFilters {
    pub kitchen_id: Option<String>,
    pub search: Option<String>,
    pub is_liked_by: Option<String>,
}

pub async fn find_many_as_user<'e, E>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsUserFilters,
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
                AND (meals.is_available = TRUE)
                AND deleted_at IS NULL
            LIMIT $2 OFFSET ($1 - 1) * $2
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
                AND (meals.is_available = TRUE)
                AND deleted_at IS NULL
        )
        SELECT 
            COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM filtered_data;
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
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

pub async fn delete_by_id_and_owner_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    owner_id: String,
) -> Result<(), Error> {
    sqlx::query!(
        "
        WITH filtered_meal AS (
            SELECT
                meals.id
            FROM
                meals,
                kitchens
            WHERE 
                kitchens.owner_id = $2
                AND meals.kitchen_id = kitchens.id
                AND meals.id = $1
        )
        UPDATE
            meals
        SET
            deleted_at = NOW()
        FROM
            filtered_meal
        WHERE
            meals.id = filtered_meal.id
        ",
        id,
        owner_id
    )
    .execute(e)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to delete a meal by id and owner_id {}: {}",
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
