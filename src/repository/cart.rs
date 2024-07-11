use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::BigDecimal;
use std::{
    convert::Into,
    ops::{Deref, DerefMut},
};
use ulid::Ulid;

use crate::repository;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CartStatus {
    CheckedOut,
    NotCheckedOut,
}

impl Into<CartStatus> for String {
    fn into(self) -> CartStatus {
        match self.as_str() {
            "CheckedOut" => CartStatus::CheckedOut,
            _ => CartStatus::NotCheckedOut,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CartItem {
    meal_id: String,
    quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CartItems {
    items: Vec<CartItem>,
}

impl Into<CartItems> for Value {
    fn into(self) -> CartItems {
        match serde_json::de::from_str::<Vec<CartItem>>(self.to_string().as_ref()) {
            Ok(items) => CartItems { items },
            Err(_) => CartItems { items: vec![] },
        }
    }
}

impl Deref for CartItems {
    type Target = Vec<CartItem>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for CartItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cart {
    pub id: String,
    pub items: CartItems,
    pub status: CartStatus,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateCartPayload {
    pub owner_id: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateCartPayload) -> Result<(), Error> {
    match sqlx::query!(
        "
        INSERT INTO carts (
            id,
            items,
            status,
            owner_id
        )
        VALUES ($1, $2, $3, $4)
    ",
        Ulid::new().to_string(),
        json!([]),
        "",
        payload.owner_id
    )
    .execute(&db.pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::info!("Error occurred while trying to create a cart: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Cart>, Error> {
    match sqlx::query_as!(
        Cart,
        "
            SELECT * FROM carts WHERE id = $1
        ",
        id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_cart) => Ok(maybe_cart),
        Err(err) => {
            tracing::info!("Error occurred while trying to fetch many carts: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Cart>,
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
) -> Result<Paginated<Cart>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM carts 
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM carts
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
            tracing::info!("Error occurred while trying to fetch many carts: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_many_by_owner_id(
    db: DatabaseConnection,
    pagination: Pagination,
    owner_id: String,
) -> Result<Paginated<Cart>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM carts 
                WHERE owner_id = $3
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM carts
            )
            SELECT JSONB_BUILD_OBJECT(
                'data', JSONB_AGG(ROW_TO_JSON(filtered_data)),
                'total', (SELECT total_rows FROM total_count)
            ) AS result
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
        owner_id,
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
            tracing::info!("Error occurred while trying to fetch many carts: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Serialize)]
pub struct UpdateCartPayload {
    pub items: Option<CartItems>,
    pub status: Option<String>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateCartPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE carts SET
                items = COALESCE($1, items),
                status = COALESCE($2, status),
                updated_at = NOW()
            WHERE
                id = $3
        ",
        json!(payload.items),
        payload.status,
        id.clone(),
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!(
                "Error occurred while trying to update cart by id {}: {}",
                id,
                e
            );
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}

pub fn is_owner(user: repository::user::User, cart: Cart) -> bool {
    cart.owner_id == user.id
}
