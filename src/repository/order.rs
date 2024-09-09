use super::meal::Meal;
use super::user::User;
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use std::{convert::Into, str::FromStr};
use ulid::Ulid;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

use super::cart::{self, Cart};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderStatus {
    #[serde(rename = "AWAITING_PAYMENT")]
    AwaitingPayment,
    #[serde(rename = "AWAITING_ACKNOWLEDGEMENT")]
    AwaitingAcknowledgement,
    #[serde(rename = "PREPARING")]
    Preparing,
    #[serde(rename = "IN_TRANSIT")]
    InTransit,
    #[serde(rename = "DELIVERED")]
    Delivered,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}

impl ToString for OrderStatus {
    fn to_string(&self) -> String {
        match self {
            OrderStatus::AwaitingPayment => String::from("AWAITING_PAYMENT"),
            OrderStatus::AwaitingAcknowledgement => String::from("AWAITING_ACKNOWLEDGEMENT"),
            OrderStatus::Preparing => String::from("PREPARING"),
            OrderStatus::InTransit => String::from("IN_TRANSIT"),
            OrderStatus::Delivered => String::from("DELIVERED"),
            OrderStatus::Cancelled => String::from("CANCELLED"),
        }
    }
}

impl FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AWAITING_PAYMENT" => Ok(OrderStatus::AwaitingPayment),
            "AWAITING_ACKNOWLEDGEMENT" => Ok(OrderStatus::AwaitingAcknowledgement),
            "PREPARING" => Ok(OrderStatus::Preparing),
            "IN_TRANSIT" => Ok(OrderStatus::InTransit),
            "DELIVERED" => Ok(OrderStatus::Delivered),
            "CANCELLED" => Ok(OrderStatus::Cancelled),
            _ => Err(format!("'{}' is not a valid OrderStatus", s)),
        }
    }
}

impl From<String> for OrderStatus {
    fn from(s: String) -> Self {
        s.parse()
            .unwrap_or_else(|_| panic!("Failed to parse '{}' into an OrderStatus", s))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PaymentMethod {
    #[serde(rename = "ONLINE")]
    Online,
    #[serde(rename = "WALLET")]
    Wallet,
}

impl ToString for PaymentMethod {
    fn to_string(&self) -> String {
        match self {
            PaymentMethod::Online => String::from("ONLINE"),
            PaymentMethod::Wallet => String::from("WALLET"),
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ONLINE" => Ok(PaymentMethod::Online),
            "WALLET" => Ok(PaymentMethod::Wallet),
            _ => Err(format!("'{}' is not a valid PaymentMethod", s)),
        }
    }
}

impl From<String> for PaymentMethod {
    fn from(s: String) -> Self {
        s.parse()
            .unwrap_or_else(|_| panic!("Failed to parse '{}' into an PaymentMethod", s))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Order {
    pub id: String,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub delivery_fee: BigDecimal,
    pub service_fee: BigDecimal,
    pub sub_total: BigDecimal,
    pub total: BigDecimal,
    pub delivery_address: String,
    pub dispatch_rider_note: String,
    pub cart_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderItem {
    pub id: i32,
    pub status: OrderStatus,
    pub price: BigDecimal,
    pub meal_id: String,
    pub order_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderUpdate {
    pub id: i32,
    pub status: OrderStatus,
    pub order_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct CreateOrderPayload {
    pub cart: Cart,
    pub payment_method: PaymentMethod,
    pub delivery_address: String,
    pub dispatch_rider_note: String,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateOrderPayload) -> Result<Order, Error> {
    // TODO: are these fields for fancy?
    let delivery_fee = BigDecimal::from_i32(0).unwrap();
    let service_fee = BigDecimal::from_i32(0).unwrap();
    let meals = match cart::get_meals_from_cart_by_id(db.clone(), payload.cart.id.clone()).await {
        Ok(meals) => meals,
        Err(_) => return Err(Error::UnexpectedError),
    };
    let meal_ids = meals
        .clone()
        .into_iter()
        .map(|meal| meal.id)
        .collect::<Vec<_>>();
    let sub_total = meals
        .clone()
        .into_iter()
        .map(|meal| meal.price)
        .reduce(|acc, price| acc + price)
        .unwrap();
    let total = sub_total.clone();

    // FIX: should make use of a transaction

    let order = match sqlx::query_as!(
        Order,
        "
        INSERT INTO orders (
            id,
            status,
            payment_method,
            delivery_fee,
            service_fee,
            sub_total,
            total,
            delivery_address,
            dispatch_rider_note,
            cart_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        ",
        Ulid::new().to_string(),
        OrderStatus::AwaitingPayment.to_string(),
        payload.payment_method.to_string(),
        delivery_fee,
        service_fee,
        sub_total,
        total,
        payload.delivery_address,
        payload.dispatch_rider_note,
        payload.cart.id
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(order) => order,
        Err(err) => {
            tracing::error!("Error occurred while trying to create a order: {}", err);
            return Err(Error::UnexpectedError);
        }
    };

    match sqlx::query!("
        WITH cte_meals AS (SELECT * FROM JSONB_ARRAY_ELEMENTS($1) AS meal_id LEFT JOIN meals ON meal_id#>>'{}' = meals.id)
        INSERT INTO order_items (status, price, meal_id, order_id)
        SELECT $2, cte_meals.price, cte_meals.id, $3 FROM cte_meals;
    ",
        json!(meal_ids), OrderStatus::AwaitingPayment.to_string(), order.id
    )
    .execute(&db.pool)
    .await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("Failed to create order items from meal ids {:?}: {}", meal_ids, err);
            return Err(Error::UnexpectedError);
        }
    };

    Ok(order)
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Order>, Error> {
    match sqlx::query_as!(Order, "SELECT * FROM orders WHERE id = $1", id)
        .fetch_optional(&db.pool)
        .await
    {
        Ok(maybe_order) => Ok(maybe_order),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch order by id: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id_and_owner_id(
    db: DatabaseConnection,
    id: String,
    owner_id: String,
) -> Result<Option<Order>, Error> {
    match sqlx::query_as!(
        Order,
        "
        SELECT orders.* FROM orders
        LEFT JOIN carts ON orders.cart_id = carts.id
        WHERE orders.id = $1 AND owner_id = $2
        ",
        id,
        owner_id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_order) => Ok(maybe_order),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch order by id: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_order_items_by_id(
    db: DatabaseConnection,
    id: String,
) -> Result<Vec<OrderItem>, Error> {
    match sqlx::query_as!(
        OrderItem,
        "SELECT * FROM order_items WHERE order_id = $1",
        id
    )
    .fetch_all(&db.pool)
    .await
    {
        Ok(items) => Ok(items),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch many order items by id {}: {}",
                id,
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Order>,
    total: u32,
}

impl Into<DatabaseCountedResult> for Option<serde_json::Value> {
    fn into(self) -> DatabaseCountedResult {
        match self {
            Some(json) => {
                tracing::info!("About to deserialize: {}", json);
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
) -> Result<Paginated<Order>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM orders 
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM orders
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
            tracing::error!("Error occurred while trying to fetch many orders: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_many_by_owner_id(
    db: DatabaseConnection,
    owner_id: String,
    pagination: Pagination,
) -> Result<Paginated<Order>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT orders.*
                FROM orders 
                LEFT JOIN carts ON cart_id = carts.id
                WHERE owner_id = $3
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM orders
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
            tracing::error!("Error occurred while trying to fetch many orders: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Serialize)]
pub struct UpdateOrderPayload {
    pub status: OrderStatus,
}

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateOrderPayload,
) -> Result<(), Error> {
    // FIX: this should make use of a transaction

    if let Err(err) = sqlx::query!(
        "INSERT INTO order_updates (status, order_id) VALUES ($1, $2)",
        payload.status.to_string(),
        id
    )
    .execute(&db.pool)
    .await
    {
        tracing::error!(
            "Error occurred while trying to insert bookkeeping records for order updates: {}",
            err
        );
        return Err(Error::UnexpectedError);
    }

    match sqlx::query!(
        "
            UPDATE orders SET
                status = $1,
                updated_at = NOW()
            WHERE
                id = $2
        ",
        payload.status.to_string(),
        id,
    )
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!("Error occurred while trying to update order by id: {}", e);
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}

pub async fn get_meals_from_order_by_id(
    db: DatabaseConnection,
    id: String,
) -> Result<Vec<Meal>, Error> {
    match find_order_items_by_id(db.clone(), id.clone()).await {
        Ok(items) => {
            let meal_ids = items
                .iter()
                .map(|item| item.meal_id.clone())
                .collect::<Vec<_>>();

            match sqlx::query_as!(
                Meal,
                "SELECT * FROM meals WHERE $1 @> TO_JSONB(meals.id)",
                json!(meal_ids),
            )
            .fetch_all(&db.pool)
            .await
            {
                Ok(meals) => Ok(meals),
                Err(e) => {
                    log::error!(
                        "Error occurred while trying to get meals from order by id {}: {}",
                        id,
                        e
                    );
                    Err(Error::UnexpectedError)
                }
            }
        }
        Err(e) => {
            log::error!(
                "Error occurred while trying to get meals from order by id {}: {:?}",
                id,
                e
            );
            Err(Error::UnexpectedError)
        }
    }
}
