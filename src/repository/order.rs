use super::meal::Meal;
use super::user::User;
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Json;
use sqlx::{types::BigDecimal, Database};
use std::{convert::Into, str::FromStr};
use ulid::Ulid;

use crate::{
    define_paginated,
    utils::{
        database::DatabaseConnection,
        pagination::{Paginated, Pagination},
    },
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

#[derive(Clone, Debug, Deserialize)]
pub enum OrderSimpleStatus {
    #[serde(rename = "ONGOING")]
    Ongoing,
    #[serde(rename = "COMPLETED")]
    Completed,
}

impl ToString for OrderSimpleStatus {
    fn to_string(&self) -> String {
        match self {
            OrderSimpleStatus::Ongoing => String::from("ONGOING"),
            OrderSimpleStatus::Completed => String::from("COMPLETED"),
        }
    }
}

impl FromStr for OrderSimpleStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ONGOING" => Ok(OrderSimpleStatus::Ongoing),
            "COMPLETED" => Ok(OrderSimpleStatus::Completed),
            _ => Err(format!("'{}' is not a valid OrderSimpleStatus", s)),
        }
    }
}

impl From<String> for OrderSimpleStatus {
    fn from(s: String) -> Self {
        s.parse()
            .unwrap_or_else(|_| panic!("Failed to parse '{}' into an OrderSimpleStatus", s))
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
    pub owner_id: String,
    pub cart_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OrderItems(pub Vec<OrderItem>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderItem {
    pub id: String,
    pub status: OrderStatus,
    pub price: BigDecimal,
    pub meal_id: String,
    pub order_id: String,
    pub kitchen_id: String,
    pub owner_id: String,
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

define_paginated!(DatabasePaginatedOrder, Order);

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
                    cart_id,
                    owner_id
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
                    $2,
                    active_cart.owner_id
                FROM sub_total_calculation, active_cart
                RETURNING *
            ),
            inserted_items AS (
                INSERT INTO order_items (id, status, price, meal_id, order_id, kitchen_id, owner_id)
                SELECT 
                    GEN_RANDOM_UUID(),
                    $3,
                    meals_in_cart.price,
                    meals_in_cart.id,
                    inserted_order.id,
                    meals_in_cart.kitchen_id,
                    active_cart.owner_id
                FROM meals_in_cart, active_cart
                CROSS JOIN inserted_order
                RETURNING *
            )
            SELECT 
                inserted_order.*,
                COALESCE(JSON_AGG(inserted_items.*), '[]'::json) AS items
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
                inserted_order.owner_id,
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
    .map_err(|err| {
            tracing::error!("Error occurred while trying to create a order: {}", err);
            Error::UnexpectedError
    })
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Order>, Error> {
    sqlx::query_as!(
        Order,
        "
            WITH order_items AS (
                SELECT * FROM order_items WHERE order_id = $1
            )
            SELECT orders.*, JSON_AGG(order_items.*) as items
                FROM orders JOIN order_items ON order_items.order_id = orders.id
            GROUP BY
                orders.id;
        ",
        id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch order by id: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_by_id_and_owner_id(
    db: DatabaseConnection,
    id: String,
    owner_id: String,
) -> Result<Option<Order>, Error> {
    sqlx::query_as!(
        Order,
        "
            WITH order_items AS (
                SELECT * FROM order_items
                WHERE
                    order_id = $1
            )
            SELECT orders.*, JSON_AGG(order_items.*) as items
                FROM orders JOIN order_items ON order_items.order_id = orders.id
            WHERE
                orders.owner_id = $2
            GROUP BY
                orders.id;
        ",
        id,
        owner_id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch order by id: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_order_item_by_id(
    db: DatabaseConnection,
    id: String,
) -> Result<Option<OrderItem>, Error> {
    sqlx::query_as!(
        OrderItem,
        "
            SELECT * FROM order_items WHERE id = $1;
        ",
        id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch order item by id: {}",
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct Filters {
    pub owner_id: Option<String>,
    pub status: Option<OrderSimpleStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub kitchen_id: Option<String>,
}

pub async fn find_many(
    db: DatabaseConnection,
    pagination: Pagination,
    filters: Filters,
) -> Result<Paginated<Order>, Error> {
    sqlx::query_as!(
        DatabasePaginatedOrder,
        r#"
            WITH filtered_data AS (
                SELECT orders.*,
                       COALESCE(
                           JSONB_AGG(ROW_TO_JSON(order_items)) FILTER (WHERE order_items.id IS NOT NULL),
                           '[]'::jsonb
                       ) AS items
                FROM orders
                LEFT JOIN order_items ON orders.id = order_items.order_id
                WHERE
                    ($3::TEXT IS NULL OR orders.owner_id = $3)
                    AND (
                        $4::TEXT IS NULL OR 
                        CASE
                            WHEN $4 = 'ONGOING' THEN orders.status IN ('AWAITING_PAYMENT', 'AWAITING_ACKNOWLEDGEMENT', 'PREPARING', 'IN_TRANSIT')
                            WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                            ELSE TRUE
                        END
                    )
                    AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                    AND ($6::TEXT IS NULL OR order_items.kitchen_id = $6)
                GROUP BY orders.id
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(DISTINCT orders.id) AS total_rows
                FROM orders
                LEFT JOIN order_items ON orders.id = order_items.order_id
                WHERE
                    ($3::TEXT IS NULL OR orders.owner_id = $3)
                    AND (
                        $4::TEXT IS NULL OR 
                        CASE
                            WHEN $4 = 'ONGOING' THEN orders.status IN ('AWAITING_PAYMENT', 'AWAITING_ACKNOWLEDGEMENT', 'PREPARING', 'IN_TRANSIT')
                            WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                            ELSE TRUE
                        END
                    )
                    AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                    AND ($6::TEXT IS NULL OR order_items.kitchen_id = $6)
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
        filters.owner_id,
        filters.status.map(|s| s.to_string()),
        filters.payment_method.map(|p| p.to_string()),
        filters.kitchen_id
    )
    .fetch_one(&db.pool)
    .await
    .map(DatabasePaginatedOrder::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many orders: {}", err);
        Error::UnexpectedError
    })
}

pub async fn confirm_payment(db: DatabaseConnection, order_id: String) -> Result<bool, Error> {
    sqlx::query!(
        r#"
        WITH updated_order AS (
            UPDATE orders
            SET status = 'AWAITING_ACKNOWLEDGEMENT'
            WHERE id = $1
              AND status = 'AWAITING_PAYMENT'
            RETURNING id
        ),
        updated_order_items AS (
            UPDATE order_items
            SET status = 'AWAITING_ACKNOWLEDGEMENT'
            WHERE order_id = (SELECT id FROM updated_order)
            RETURNING id
        )
        SELECT EXISTS(SELECT 1 FROM updated_order);
        "#,
        order_id
    )
    .fetch_optional(&db.pool)
    .await
    .map(|opt| opt.is_some())
    .map_err(|err| {
        tracing::error!("Error confirming payment for order {}: {}", order_id, err);
        Error::UnexpectedError
    })
}

pub async fn update_order_item_status(
    db: DatabaseConnection,
    order_item_id: String,
    new_status: OrderStatus,
) -> Result<bool, Error> {
    sqlx::query!(
        r#"
        WITH valid_transition AS (
            SELECT CASE
                WHEN order_items.status = 'AWAITING_ACKNOWLEDGEMENT' AND $2 = 'PREPARING' THEN TRUE
                WHEN order_items.status = 'PREPARING' AND $2 = 'IN_TRANSIT' THEN TRUE
                WHEN order_items.status = 'IN_TRANSIT' AND $2 = 'DELIVERED' THEN TRUE
                ELSE FALSE
            END AS is_valid
            FROM order_items
            WHERE id = $1
        ),
        updated_item AS (
            UPDATE order_items
            SET status = $2
            WHERE id = $1
              AND (SELECT is_valid FROM valid_transition)
            RETURNING id
        )
        SELECT EXISTS(SELECT 1 FROM updated_item);
        "#,
        order_item_id,
        new_status.to_string()
    )
    .fetch_optional(&db.pool)
    .await
    .map(|opt| opt.is_some())
    .map_err(|err| {
        tracing::error!(
            "Error updating status for order item {}: {}",
            order_item_id,
            err
        );
        Error::UnexpectedError
    })
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
