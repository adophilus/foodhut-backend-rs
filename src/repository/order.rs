use super::meal::Meal;
use super::user::User;
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{types::BigDecimal, Database};
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
    pub items: OrderItems,
    pub cart_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OrderItems(pub Vec<OrderItem>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderItem {
    pub id: i32,
    pub status: OrderStatus,
    pub price: BigDecimal,
    pub meal_id: String,
    pub order_id: String,
    pub kitchen_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Option<serde_json::Value>> for OrderItems {
    fn from(v: Option<serde_json::Value>) -> Self {
        match v {
            Some(json) => serde_json::de::from_str::<_>(json.to_string().as_ref())
                .expect("Invalid order items list"),
            None => unreachable!(),
        }
    }
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
    // TODO: service fee and delivery fee calculation required (in query)
    sqlx::query_as!(
        Order,
        "
            WITH active_cart AS (
                SELECT * FROM carts WHERE id = $2
            ), cart_items AS (
                SELECT * FROM JSON_TO_RECORDSET((SELECT active_cart.items FROM active_cart)) AS (meal_id VARCHAR, quantity NUMERIC)
            ),
            meals_in_cart AS (
                SELECT meals.*, cart_items.quantity AS quantity FROM cart_items INNER JOIN meals ON cart_items.meal_id = meals.id
            ),
            sub_total_calculation AS (
                SELECT SUM(meals_in_cart.price * meals_in_cart.quantity) AS sub_total 
                FROM meals_in_cart
            ),
            inserted_order AS (
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
                SELECT 
                    $1,
                    $3,
                    $4,
                    0,
                    0,
                    sub_total_calculation.sub_total,
                    sub_total_calculation.sub_total + 0,
                    $5,
                    $6,
                    $2
                FROM sub_total_calculation
                RETURNING *
            ),
            inserted_items AS (
                INSERT INTO order_items (status, price, meal_id, order_id, kitchen_id)
                SELECT 
                    $3,
                    meals_in_cart.price,
                    meals_in_cart.id,
                    inserted_order.id,
                    meals_in_cart.kitchen_id
                FROM meals_in_cart
                CROSS JOIN inserted_order
                RETURNING *
            )
            SELECT 
                inserted_order.*,
                COALESCE(JSON_AGG(inserted_items.*), '[]')::json AS items
            FROM inserted_order
            LEFT JOIN inserted_items ON inserted_order.id = inserted_items.order_id
            GROUP BY 
                inserted_order.id, 
                inserted_order.status,
                inserted_order.payment_method,
                inserted_order.delivery_fee,
                inserted_order.service_fee,
                inserted_order.sub_total,
                inserted_order.total,
                inserted_order.delivery_address,
                inserted_order.dispatch_rider_note,
                inserted_order.cart_id,
                inserted_order.created_at,
                inserted_order.updated_at;
        ",
        Ulid::new().to_string(),
        payload.cart.id,
        OrderStatus::AwaitingPayment.to_string(),
        payload.payment_method.to_string(),
        // delivery_fee,
        // service_fee,
        // sub_total,
        // total,
        payload.delivery_address,
        payload.dispatch_rider_note,
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
            tracing::error!("Error occurred while trying to create a order: {}", err);
            Error::UnexpectedError
    })
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

// pub async fn find_by_kitchen_id(db: DatabaseConnection, id: String) -> Result<Vec<Order>, Error> {
//     match sqlx::query_as!(
//         Order,
//         "SELECT FROM order_items WHERE kitchen_id = $1 LEFT JOIN orders ON order_id = orders.id",
//         id
//     )
//     .fetch_all(&db.pool)
//     .await
//     {
//         Ok(orders) => Ok(orders),
//         Err(err) => {
//             tracing::error!(
//                 "Error occurred while trying to fetch many orders by kitchen id {}: {}",
//                 id,
//                 err
//             );
//             Err(Error::UnexpectedError)
//         }
//     }
// }

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
                'data', COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb),
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
                'data', COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb),
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
