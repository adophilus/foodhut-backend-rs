use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgExecutor;
use std::{
    convert::{From, Into},
    ops::{Deref, DerefMut},
};
use ulid::Ulid;

use crate::{
    define_paginated,
    modules::{kitchen::repository::Kitchen, meal::repository::Meal},
};

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

define_paginated!(DatabasePaginatedCart, Cart);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullCartItem {
    pub meal_id: String,
    pub quantity: i32,
    pub meal: Meal,
    pub kitchen: Kitchen,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullCartItems(pub Vec<FullCartItem>);

impl From<Option<serde_json::Value>> for FullCartItems {
    fn from(v: Option<serde_json::Value>) -> Self {
        match v {
            Some(json) => serde_json::de::from_str::<_>(json.to_string().as_ref())
                .expect("Invalid full cart items list"),
            None => unreachable!("Invalid full cart items list"),
        }
    }
}

impl Deref for FullCartItems {
    type Target = Vec<FullCartItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FullCartItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullCart {
    pub id: String,
    pub items: FullCartItems,
    pub status: CartStatus,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

define_paginated!(DatabasePaginatedFullCart, FullCart);

pub struct CreateCartPayload {
    pub owner_id: String,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E>(e: E, payload: CreateCartPayload) -> Result<Cart, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
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
    .fetch_one(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to create a cart: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_by_id<'e, Executor: PgExecutor<'e>>(
    e: Executor,
    id: String,
) -> Result<Option<Cart>, Error> {
    sqlx::query_as!(
        Cart,
        "
            SELECT * FROM carts WHERE id = $1
        ",
        id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many carts: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_full_cart_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
) -> Result<Option<FullCart>, Error> {
    sqlx::query_as!(
        FullCart,
        r#"
        WITH cart_data AS (
            SELECT 
                carts.id,
                carts.status,
                carts.owner_id,
                carts.created_at,
                carts.updated_at,
                COALESCE(
                    JSONB_AGG(
                        TO_JSONB(ROW_TO_JSON(cart_items)) || 
                        JSONB_BUILD_OBJECT(
                            'meal', meals
                        )
                    ) FILTER (WHERE cart_items.meal_id IS NOT NULL),
                    '[]'::jsonb
                ) AS items
            FROM carts
            LEFT JOIN LATERAL jsonb_to_recordset(carts.items::jsonb) AS cart_items(meal_id TEXT, quantity INT) ON true
            LEFT JOIN meals ON cart_items.meal_id = meals.id
            WHERE carts.id = $1
            GROUP BY carts.id
        )
        SELECT * FROM cart_data;
        "#,
        id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch cart by id {}: {}", id, err);
        Error::UnexpectedError
    })
}

pub async fn find_active_cart_by_owner_id<'e, E: PgExecutor<'e>>(
    e: E,
    owner_id: String,
) -> Result<Option<Cart>, Error> {
    sqlx::query_as!(
        Cart,
        "SELECT * FROM carts WHERE owner_id = $1 AND status = $2",
        owner_id,
        CartStatus::NotCheckedOut.to_string(),
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch cart by owner id {}: {}",
            owner_id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_active_full_cart_by_owner_id<'e, E: PgExecutor<'e>>(
    e: E,
    owner_id: String,
) -> Result<Option<FullCart>, Error> {
    sqlx::query_as!(
        FullCart,
        r#"
        WITH cart_data AS (
            SELECT 
                carts.id,
                carts.status,
                carts.owner_id,
                carts.created_at,
                carts.updated_at,
                COALESCE(
                    JSONB_AGG(
                        TO_JSONB(ROW_TO_JSON(cart_items)) || 
                        JSONB_BUILD_OBJECT(
                            'meal', meals,
                            'kitchen', kitchens
                        )
                    ) FILTER (WHERE cart_items.meal_id IS NOT NULL),
                    '[]'::jsonb
                ) AS items
            FROM carts
            LEFT JOIN LATERAL JSONB_TO_RECORDSET(carts.items::jsonb) AS cart_items(meal_id TEXT, quantity INT) ON true
            LEFT JOIN meals ON cart_items.meal_id = meals.id
            LEFT JOIN kitchens ON meals.kitchen_id = meals.kitchen_id
            WHERE carts.owner_id = $1 AND carts.status = $2
            GROUP BY carts.id
        )
        SELECT * FROM cart_data;
        "#,
        owner_id,
        CartStatus::NotCheckedOut.to_string()
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch active cart by owner id {}: {}",
            owner_id,
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Serialize)]
pub struct UpdateCartPayload {
    pub items: Option<CartItems>,
    pub status: Option<CartStatus>,
}

pub async fn update_by_id<'e, E: PgExecutor<'e>>(
    db: E,
    id: String,
    payload: UpdateCartPayload,
) -> Result<(), Error> {
    sqlx::query!(
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
    .execute(db)
    .await
    .map(|_| ())
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to update cart by id {}: {}",
            id,
            err
        );
        Error::UnexpectedError
    })
}
