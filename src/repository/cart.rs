use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    convert::{From, Into},
    ops::{Deref, DerefMut},
};
use ulid::Ulid;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

use super::meal::Meal;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CartStatus {
    #[serde(rename = "CHECKED_OUT")]
    CheckedOut,
    #[serde(rename = "NOT_CHECKED_OUT")]
    NotCheckedOut,
}

impl From<String> for CartStatus {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "CHECKED_OUT" => CartStatus::CheckedOut,
            "NOT_CHECKED_OUT" => CartStatus::NotCheckedOut,
            status => unreachable!("Invalid cart status: {}", status),
        }
    }
}

impl ToString for CartStatus {
    fn to_string(&self) -> String {
        match self {
            CartStatus::CheckedOut => String::from("CHECKED_OUT"),
            CartStatus::NotCheckedOut => String::from("NOT_CHECKED_OUT"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CartItem {
    pub meal_id: String,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CartItems(pub Vec<CartItem>);

impl Into<CartItems> for Value {
    fn into(self) -> CartItems {
        match serde_json::de::from_str::<Vec<CartItem>>(self.to_string().as_ref()) {
            Ok(items) => CartItems(items),
            Err(_) => CartItems(vec![]),
        }
    }
}

impl Deref for CartItems {
    type Target = Vec<CartItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CartItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateCartPayload) -> Result<Cart, Error> {
    match sqlx::query_as!(
        Cart,
        "
        INSERT INTO carts (
            id,
            items,
            status,
            owner_id
        )
        VALUES ($1, $2, $3, $4)
        RETURNING *
    ",
        Ulid::new().to_string(),
        json!([]),
        CartStatus::NotCheckedOut.to_string(),
        payload.owner_id
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(cart) => Ok(cart),
        Err(err) => {
            tracing::error!("Error occurred while trying to create a cart: {}", err);
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
            tracing::error!("Error occurred while trying to fetch many carts: {}", err);
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
            tracing::error!("Error occurred while trying to fetch many carts: {}", err);
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
            tracing::error!("Error occurred while trying to fetch many carts: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Serialize)]
pub struct UpdateCartPayload {
    pub items: Option<CartItems>,
    pub status: Option<CartStatus>,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateCartPayload,
) -> Result<(), Error> {
    match sqlx::query!(
        "
            UPDATE carts SET
                items = COALESCE(
                    CASE WHEN $1::text = 'null' THEN NULL ELSE $1::json END, 
                    items
                ),
                status = COALESCE($2, status),
                updated_at = NOW()
            WHERE
                id = $3
        ",
        json!(payload.items).to_string(),
        payload.status.map(|p| p.to_string()),
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
            Err(Error::UnexpectedError)
        }
        _ => Ok(()),
    }
}

pub async fn get_meals_from_cart_by_id(
    db: DatabaseConnection,
    id: String,
) -> Result<Vec<Meal>, Error> {
    match find_by_id(db.clone(), id.clone()).await {
        Ok(Some(cart)) => {
            let meal_ids = cart
                .items
                .iter()
                .map(|item| item.meal_id.clone())
                .collect::<Vec<_>>();
            match sqlx::query_as!(
                Meal,
                "SELECT * FROM meals WHERE $1::jsonb @> meals.id::jsonb",
                json!(meal_ids),
            )
            .fetch_all(&db.pool)
            .await
            {
                Ok(meals) => Ok(meals),
                Err(e) => {
                    log::error!(
                        "Error occurred while trying to get meals from cart by id {}: {}",
                        id,
                        e
                    );
                    Err(Error::UnexpectedError)
                }
            }
        }
        Ok(None) => Err(Error::UnexpectedError),
        Err(e) => {
            log::error!(
                "Error occurred while trying to get meals from cart by id {}: {:?}",
                id,
                e
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub fn is_owner(user: super::user::User, cart: Cart) -> bool {
    cart.owner_id == user.id
}
