use crate::modules::{payment, user::repository::User};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use sqlx::PgExecutor;
use std::{convert::Into, str::FromStr};
use ulid::Ulid;

use crate::{
    define_paginated,
    modules::{
        cart::repository::FullCartItem, kitchen::repository::Kitchen, meal::repository::Meal,
    },
    utils::pagination::{Paginated, Pagination},
};

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
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "ONGOING")]
    Ongoing,
    #[serde(rename = "COMPLETED")]
    Completed,
}

impl ToString for OrderSimpleStatus {
    fn to_string(&self) -> String {
        match self {
            OrderSimpleStatus::Pending => String::from("PENDING"),
            OrderSimpleStatus::Ongoing => String::from("ONGOING"),
            OrderSimpleStatus::Completed => String::from("COMPLETED"),
        }
    }
}

impl FromStr for OrderSimpleStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(OrderSimpleStatus::Pending),
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

impl From<payment::service::PaymentMethod> for PaymentMethod {
    fn from(method: payment::service::PaymentMethod) -> Self {
        match method {
            payment::service::PaymentMethod::Online => PaymentMethod::Online,
            payment::service::PaymentMethod::Wallet => PaymentMethod::Wallet,
        }
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
    pub delivery_date: Option<NaiveDateTime>,
    pub dispatch_rider_note: String,
    pub items: OrderItems,
    pub kitchen_id: String,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderItems(pub Vec<OrderItem>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderItem {
    pub price: BigDecimal,
    pub quantity: i32,
    pub meal_id: String,
}

impl From<serde_json::Value> for OrderItems {
    fn from(json: serde_json::Value) -> Self {
        serde_json::de::from_str::<_>(json.to_string().as_ref()).expect("Invalid order items list")
    }
}

impl From<Option<serde_json::Value>> for OrderItems {
    fn from(v: Option<serde_json::Value>) -> Self {
        match v {
            Some(json) => serde_json::de::from_str::<_>(json.to_string().as_ref())
                .expect("Invalid order items list"),
            None => unreachable!("Invalid order iems list"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullOrder {
    pub id: String,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub delivery_fee: BigDecimal,
    pub service_fee: BigDecimal,
    pub sub_total: BigDecimal,
    pub total: BigDecimal,
    pub delivery_address: String,
    pub delivery_date: Option<NaiveDateTime>,
    pub dispatch_rider_note: String,
    pub items: FullOrderItems,
    pub kitchen: Kitchen,
    pub kitchen_id: String,
    // pub owner: User,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullOrderWithOwner {
    pub id: String,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub delivery_fee: BigDecimal,
    pub service_fee: BigDecimal,
    pub sub_total: BigDecimal,
    pub total: BigDecimal,
    pub delivery_address: String,
    pub delivery_date: Option<NaiveDateTime>,
    pub dispatch_rider_note: String,
    pub items: FullOrderItems,
    pub kitchen: Kitchen,
    pub kitchen_id: String,
    pub owner: User,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}


impl FullOrder {
    pub fn add_owner(self: Self, owner: User) -> FullOrderWithOwner {
        FullOrderWithOwner {
            id: self.id,
            status: self.status,
            payment_method: self.payment_method,
            delivery_fee: self.delivery_fee,
            service_fee: self.service_fee,
            sub_total: self.sub_total,
            total: self.total,
            delivery_address: self.delivery_address,
            delivery_date: self.delivery_date,
            dispatch_rider_note: self.dispatch_rider_note,
            items: self.items,
            kitchen: self.kitchen,
            kitchen_id: self.kitchen_id,
            owner,
            owner_id: self.owner_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullOrderItems(pub Vec<FullOrderItem>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullOrderItem {
    pub price: BigDecimal,
    pub quantity: i32,
    pub meal_id: String,
    pub meal: Meal,
}

impl From<Option<serde_json::Value>> for FullOrderItems {
    fn from(v: Option<serde_json::Value>) -> Self {
        match v {
            Some(json) => serde_json::de::from_str::<_>(json.to_string().as_ref())
                .expect("Invalid full order items list"),
            None => unreachable!("Invalid full order items list"),
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
define_paginated!(DatabasePaginatedFullOrder, FullOrder);

pub struct CreateOrderPayload {
    pub items: Vec<FullCartItem>,
    pub payment_method: PaymentMethod,
    pub delivery_address: String,
    pub delivery_date: Option<NaiveDateTime>,
    pub dispatch_rider_note: String,
    pub kitchen_id: String,
    pub owner_id: String,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreateOrderPayload,
) -> Result<Order, Error> {
    let sub_total = payload
        .items
        .clone()
        .into_iter()
        .fold(BigDecimal::from(0), |acc, item| acc + (item.meal.price * BigDecimal::from(item.quantity)));
    let total = sub_total.clone();

    let order_items = OrderItems(
        payload
            .items
            .into_iter()
            .map(|item| OrderItem {
                price: item.meal.price,
                quantity: item.quantity,
                meal_id: item.meal.id,
            })
            .collect::<Vec<OrderItem>>(),
    );

    sqlx::query_as!(
        Order,
        r#"
        INSERT INTO orders (
            id,
            status,
            payment_method,
            delivery_fee,
            service_fee,
            sub_total,
            total,
            delivery_address,
            delivery_date,
            dispatch_rider_note,
            items,
            kitchen_id,
            owner_id
        )
        VALUES (
            $1,
            $2,
            $3,
            0,
            0,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11
        )
        RETURNING *
        "#,
        Ulid::new().to_string(),
        OrderStatus::AwaitingPayment.to_string(),
        payload.payment_method.to_string(),
        sub_total,
        total,
        payload.delivery_address,
        payload.delivery_date,
        payload.dispatch_rider_note,
        json!(order_items),
        payload.kitchen_id,
        payload.owner_id,
    )
    .fetch_one(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to create a order: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Option<Order>, Error> {
    sqlx::query_as!(
        Order,
        "
        SELECT * FROM orders WHERE id = $1
        ",
        id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch order by id: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_full_order_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    order_id: String,
) -> Result<Option<FullOrder>, Error> {
    sqlx::query_as!(
        FullOrder,
        r#"
        WITH filtered_orders AS (
            SELECT
                orders.id,
                orders.status,
                orders.payment_method,
                orders.delivery_fee,
                orders.service_fee,
                orders.sub_total,
                orders.total,
                orders.delivery_address,
                orders.delivery_date,
                orders.dispatch_rider_note,
                orders.kitchen_id,
                orders.owner_id,
                orders.created_at,
                orders.updated_at,
                json_item AS item
            FROM
                orders,
                JSON_ARRAY_ELEMENTS(orders.items) AS json_item
            WHERE
                orders.id = $1
        ),
        order_with_item AS (
            SELECT
                filtered_orders.id,
                filtered_orders.status,
                filtered_orders.payment_method,
                filtered_orders.delivery_fee,
                filtered_orders.service_fee,
                filtered_orders.sub_total,
                filtered_orders.total,
                filtered_orders.delivery_address,
                filtered_orders.delivery_date,
                filtered_orders.dispatch_rider_note,
                filtered_orders.kitchen_id,
                filtered_orders.owner_id,
                filtered_orders.created_at,
                filtered_orders.updated_at,
                filtered_orders.item::JSONB || JSONB_BUILD_OBJECT(
                    'meal', meals
                ) AS item,
                TO_JSONB(kitchens) || JSONB_BUILD_OBJECT('city', kitchen_cities) AS kitchen,
                TO_JSONB(users) AS owner
            FROM
                filtered_orders
            INNER JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            INNER JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
            INNER JOIN
                kitchen_cities
            ON kitchen_cities.id = kitchens.city_id
            INNER JOIN
                users
            ON users.id = filtered_orders.owner_id
        )
        SELECT
            order_with_item.id,
            order_with_item.status,
            order_with_item.payment_method,
            order_with_item.delivery_fee,
            order_with_item.service_fee,
            order_with_item.sub_total,
            order_with_item.total,
            order_with_item.delivery_address,
            order_with_item.delivery_date,
            order_with_item.dispatch_rider_note,
            order_with_item.kitchen_id,
            order_with_item.kitchen AS "kitchen!: sqlx::types::Json<Kitchen>",
            order_with_item.owner_id,
            -- order_with_item.owner AS "owner!: sqlx::types::Json<User>",
            order_with_item.created_at,
            order_with_item.updated_at,
            JSON_AGG(item) AS items
        FROM
            order_with_item
        GROUP BY
            order_with_item.id,
            order_with_item.status,
            order_with_item.payment_method,
            order_with_item.delivery_fee,
            order_with_item.service_fee,
            order_with_item.sub_total,
            order_with_item.total,
            order_with_item.delivery_address,
            order_with_item.delivery_date,
            order_with_item.dispatch_rider_note,
            order_with_item.kitchen_id,
            order_with_item.kitchen,
            order_with_item.owner_id,
            -- order_with_item.owner,
            order_with_item.created_at,
            order_with_item.updated_at
        "#,
        order_id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch full order by id {}: {}",
            order_id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_full_order_by_id_and_owner_id<'e, E: PgExecutor<'e>>(
    e: E,
    order_id: String,
    owner_id: String,
) -> Result<Option<FullOrder>, Error> {
    sqlx::query_as!(
        FullOrder,
        r#"
        WITH filtered_orders AS (
            SELECT
                orders.id,
                orders.status,
                orders.payment_method,
                orders.delivery_fee,
                orders.service_fee,
                orders.sub_total,
                orders.total,
                orders.delivery_address,
                orders.delivery_date,
                orders.dispatch_rider_note,
                orders.kitchen_id,
                orders.owner_id,
                orders.created_at,
                orders.updated_at,
                json_item AS item
            FROM
                orders,
                JSON_ARRAY_ELEMENTS(orders.items) AS json_item
            WHERE
                orders.id = $1
                AND orders.owner_id = $2
        ),
        order_with_item AS (
            SELECT
                filtered_orders.id,
                filtered_orders.status,
                filtered_orders.payment_method,
                filtered_orders.delivery_fee,
                filtered_orders.service_fee,
                filtered_orders.sub_total,
                filtered_orders.total,
                filtered_orders.delivery_address,
                filtered_orders.delivery_date,
                filtered_orders.dispatch_rider_note,
                filtered_orders.kitchen_id,
                filtered_orders.owner_id,
                filtered_orders.created_at,
                filtered_orders.updated_at,
                filtered_orders.item::JSONB || JSONB_BUILD_OBJECT(
                    'meal', meals
                ) AS item,
                TO_JSONB(kitchens) || JSONB_BUILD_OBJECT('city', kitchen_cities) AS kitchen,
                TO_JSONB(users) AS owner
            FROM
                filtered_orders
            INNER JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            INNER JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
            INNER JOIN
                kitchen_cities
            ON kitchen_cities.id = kitchens.city_id
            INNER JOIN
                users
            ON users.id = filtered_orders.owner_id
        )
        SELECT
            order_with_item.id,
            order_with_item.status,
            order_with_item.payment_method,
            order_with_item.delivery_fee,
            order_with_item.service_fee,
            order_with_item.sub_total,
            order_with_item.total,
            order_with_item.delivery_address,
            order_with_item.delivery_date,
            order_with_item.dispatch_rider_note,
            order_with_item.kitchen_id,
            order_with_item.kitchen AS "kitchen!: sqlx::types::Json<Kitchen>",
            order_with_item.owner_id,
            -- order_with_item.owner AS "owner!: sqlx::types::Json<User>",
            order_with_item.created_at,
            order_with_item.updated_at,
            JSON_AGG(item) AS items
        FROM
            order_with_item
        GROUP BY
            order_with_item.id,
            order_with_item.status,
            order_with_item.payment_method,
            order_with_item.delivery_fee,
            order_with_item.service_fee,
            order_with_item.sub_total,
            order_with_item.total,
            order_with_item.delivery_address,
            order_with_item.delivery_date,
            order_with_item.dispatch_rider_note,
            order_with_item.kitchen_id,
            order_with_item.kitchen,
            order_with_item.owner_id,
            -- order_with_item.owner,
            order_with_item.created_at,
            order_with_item.updated_at
        "#,
        order_id,
        owner_id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch full order by id {} and owner id {}: {}",
            order_id,
            owner_id,
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct FindManyAsUserFilters {
    pub owner_id: Option<String>,
    pub status: Option<OrderSimpleStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub kitchen_id: Option<String>,
}

pub async fn find_many_as_user<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsUserFilters,
) -> Result<Paginated<FullOrder>, Error> {
    sqlx::query_as!(
        DatabasePaginatedFullOrder,
        r#"
        WITH filtered_orders AS (
            SELECT
                orders.id,
                orders.status,
                orders.payment_method,
                orders.delivery_fee,
                orders.service_fee,
                orders.sub_total,
                orders.total,
                orders.delivery_address,
                orders.delivery_date,
                orders.dispatch_rider_note,
                orders.kitchen_id,
                orders.owner_id,
                orders.created_at,
                orders.updated_at,
                json_item AS item
            FROM
                orders,
                JSON_ARRAY_ELEMENTS(orders.items) AS json_item
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
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
            LIMIT $2
            OFFSET ($1 - 1) * $2
        ),
        order_with_item AS (
            SELECT
                filtered_orders.id,
                filtered_orders.status,
                filtered_orders.payment_method,
                filtered_orders.delivery_fee,
                filtered_orders.service_fee,
                filtered_orders.sub_total,
                filtered_orders.total,
                filtered_orders.delivery_address,
                filtered_orders.delivery_date,
                filtered_orders.dispatch_rider_note,
                filtered_orders.kitchen_id,
                filtered_orders.owner_id,
                filtered_orders.created_at,
                filtered_orders.updated_at,
                filtered_orders.item::JSONB || JSONB_BUILD_OBJECT(
                    'meal', meals
                ) AS item,
                TO_JSONB(kitchens) || JSONB_BUILD_OBJECT('city', kitchen_cities) AS kitchen
            FROM
                filtered_orders
            INNER JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            INNER JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
            INNER JOIN
                kitchen_cities
            ON kitchen_cities.id = kitchens.city_id
        ),
        query_result AS (
            SELECT
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.created_at,
                order_with_item.updated_at,
                JSON_AGG(item) AS items
            FROM
                order_with_item
            GROUP BY
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.created_at,
                order_with_item.updated_at
        ),
        total_count AS (
            SELECT COUNT(id) AS total_rows
            FROM orders
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
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
        )
        SELECT
            COALESCE(JSONB_AGG(query_result), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM
            query_result,
            total_count
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
        filters.owner_id,
        filters.status.map(|s| s.to_string()),
        filters.payment_method.map(|p| p.to_string()),
        filters.kitchen_id
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedFullOrder::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many orders: {}", err);
        Error::UnexpectedError
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct FindManyAsKitchenFilters {
    pub owner_id: Option<String>,
    pub status: Option<OrderSimpleStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub kitchen_id: Option<String>,
}

pub async fn find_many_as_kitchen<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsKitchenFilters,
) -> Result<Paginated<FullOrder>, Error> {
    sqlx::query_as!(
        DatabasePaginatedFullOrder,
        r#"
        WITH filtered_orders AS (
            SELECT
                orders.id,
                orders.status,
                orders.payment_method,
                orders.delivery_fee,
                orders.service_fee,
                orders.sub_total,
                orders.total,
                orders.delivery_address,
                orders.delivery_date,
                orders.dispatch_rider_note,
                orders.kitchen_id,
                orders.owner_id,
                orders.created_at,
                orders.updated_at,
                json_item AS item
            FROM
                orders,
                JSON_ARRAY_ELEMENTS(orders.items) AS json_item
            WHERE
                ($3::TEXT IS NULL OR orders.owner_id = $3)
                AND (
                    $4::TEXT IS NULL OR
                    CASE
                        WHEN $4 = 'PENDING' THEN orders.status IN ('AWAITING_ACKNOWLEDGEMENT')
                        WHEN $4 = 'ONGOING' THEN orders.status IN ('PREPARING', 'IN_TRANSIT')
                        WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                        ELSE TRUE
                    END
                )
                AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
            LIMIT $2
            OFFSET ($1 - 1) * $2
        ),
        order_with_item AS (
            SELECT
                filtered_orders.id,
                filtered_orders.status,
                filtered_orders.payment_method,
                filtered_orders.delivery_fee,
                filtered_orders.service_fee,
                filtered_orders.sub_total,
                filtered_orders.total,
                filtered_orders.delivery_address,
                filtered_orders.delivery_date,
                filtered_orders.dispatch_rider_note,
                filtered_orders.kitchen_id,
                filtered_orders.owner_id,
                filtered_orders.created_at,
                filtered_orders.updated_at,
                filtered_orders.item::JSONB || JSONB_BUILD_OBJECT(
                    'meal', meals
                ) AS item,
                TO_JSONB(kitchens) || JSONB_BUILD_OBJECT('city', kitchen_cities) AS kitchen,
                TO_JSONB(users) AS owner
            FROM
                filtered_orders
            INNER JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            INNER JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
            INNER JOIN
                kitchen_cities
            ON kitchen_cities.id = kitchens.city_id
            INNER JOIN
                users
            ON users.id = filtered_orders.owner_id
        ),
        query_result AS (
            SELECT
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.created_at,
                order_with_item.updated_at,
                JSON_AGG(item) AS items
            FROM
                order_with_item
            GROUP BY
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.created_at,
                order_with_item.updated_at
        ),
        total_count AS (
            SELECT COUNT(id) AS total_rows
            FROM orders
            WHERE
                ($3::TEXT IS NULL OR orders.owner_id = $3)
                AND (
                    $4::TEXT IS NULL OR
                    CASE
                        WHEN $4 = 'PENDING' THEN orders.status IN ('AWAITING_ACKNOWLEDGEMENT')
                        WHEN $4 = 'ONGOING' THEN orders.status IN ('PREPARING', 'IN_TRANSIT')
                        WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                        ELSE TRUE
                    END
                )
                AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
        )
        SELECT
            COALESCE(JSONB_AGG(query_result), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM
            query_result,
            total_count
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
        filters.owner_id,
        filters.status.map(|s| s.to_string()),
        filters.payment_method.map(|p| p.to_string()),
        filters.kitchen_id
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedFullOrder::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many orders: {}", err);
        Error::UnexpectedError
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct FindManyAsAdminFilters {
    pub owner_id: Option<String>,
    pub status: Option<OrderSimpleStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub kitchen_id: Option<String>,
}

pub async fn find_many_as_admin<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: FindManyAsAdminFilters,
) -> Result<Paginated<FullOrder>, Error> {
    sqlx::query_as!(
        DatabasePaginatedFullOrder,
        r#"
        WITH filtered_orders AS (
            SELECT
                orders.id,
                orders.status,
                orders.payment_method,
                orders.delivery_fee,
                orders.service_fee,
                orders.sub_total,
                orders.total,
                orders.delivery_address,
                orders.delivery_date,
                orders.dispatch_rider_note,
                orders.kitchen_id,
                orders.owner_id,
                orders.created_at,
                orders.updated_at,
                json_item AS item
            FROM
                orders,
                JSON_ARRAY_ELEMENTS(orders.items) AS json_item
            WHERE
                ($3::TEXT IS NULL OR orders.owner_id = $3)
                AND (
                    $4::TEXT IS NULL OR
                    CASE
                        WHEN $4 = 'PENDING' THEN orders.status IN ('AWAITING_PAYMENT', 'AWAITING_ACKNOWLEDGEMENT')
                        WHEN $4 = 'ONGOING' THEN orders.status IN ('PREPARING', 'IN_TRANSIT')
                        WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                        ELSE TRUE
                    END
                )
                AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
            LIMIT $2
            OFFSET ($1 - 1) * $2
        ),
        order_with_item AS (
            SELECT
                filtered_orders.id,
                filtered_orders.status,
                filtered_orders.payment_method,
                filtered_orders.delivery_fee,
                filtered_orders.service_fee,
                filtered_orders.sub_total,
                filtered_orders.total,
                filtered_orders.delivery_address,
                filtered_orders.delivery_date,
                filtered_orders.dispatch_rider_note,
                filtered_orders.kitchen_id,
                filtered_orders.owner_id,
                filtered_orders.created_at,
                filtered_orders.updated_at,
                filtered_orders.item::JSONB || JSONB_BUILD_OBJECT(
                    'meal', meals
                ) AS item,
                TO_JSONB(kitchens) || JSONB_BUILD_OBJECT('city', kitchen_cities) AS kitchen,
                TO_JSONB(users) AS owner
            FROM
                filtered_orders
            INNER JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            INNER JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
            INNER JOIN
                kitchen_cities
            ON kitchen_cities.id = kitchens.city_id
            INNER JOIN
                users
            ON users.id = filtered_orders.owner_id
        ),
        query_result AS (
            SELECT
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.owner,
                order_with_item.created_at,
                order_with_item.updated_at,
                JSON_AGG(item) AS items
            FROM
                order_with_item
            GROUP BY
                order_with_item.id,
                order_with_item.status,
                order_with_item.payment_method,
                order_with_item.delivery_fee,
                order_with_item.service_fee,
                order_with_item.sub_total,
                order_with_item.total,
                order_with_item.delivery_address,
                order_with_item.delivery_date,
                order_with_item.dispatch_rider_note,
                order_with_item.kitchen_id,
                order_with_item.kitchen,
                order_with_item.owner_id,
                order_with_item.owner,
                order_with_item.created_at,
                order_with_item.updated_at
        ),
        total_count AS (
            SELECT COUNT(id) AS total_rows
            FROM orders
            WHERE
                ($3::TEXT IS NULL OR orders.owner_id = $3)
                AND (
                    $4::TEXT IS NULL OR
                    CASE
                        WHEN $4 = 'PENDING' THEN orders.status IN ('AWAITING_PAYMENT', 'AWAITING_ACKNOWLEDGEMENT')
                        WHEN $4 = 'ONGOING' THEN orders.status IN ('PREPARING', 'IN_TRANSIT')
                        WHEN $4 = 'COMPLETED' THEN orders.status IN ('DELIVERED', 'CANCELLED')
                        ELSE TRUE
                    END
                )
                AND ($5::TEXT IS NULL OR orders.payment_method = $5)
                AND ($6::TEXT IS NULL OR orders.kitchen_id = $6)
        )
        SELECT
            COALESCE(JSONB_AGG(query_result), '[]'::JSONB) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM
            query_result,
            total_count
        "#,
        pagination.page as i32,
        pagination.per_page as i32,
        filters.owner_id,
        filters.status.map(|s| s.to_string()),
        filters.payment_method.map(|p| p.to_string()),
        filters.kitchen_id
    )
    .fetch_one(e)
    .await
    .map(DatabasePaginatedFullOrder::into)
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch many orders: {}", err);
        Error::UnexpectedError
    })
}

pub struct ConfirmPaymentPayload {
    pub payment_method: PaymentMethod,
    pub order_id: String,
}

pub async fn confirm_payment<'e, E: PgExecutor<'e>>(
    e: E,
    payload: ConfirmPaymentPayload,
) -> Result<bool, Error> {
    sqlx::query!(
        r#"
        UPDATE orders
        SET
            status = 'AWAITING_ACKNOWLEDGEMENT',
            payment_method = $1
        WHERE
            id = $2
            AND status = 'AWAITING_PAYMENT'
        "#,
        payload.payment_method.to_string(),
        payload.order_id,
    )
    .fetch_optional(e)
    .await
    .map(|opt| opt.is_some())
    .map_err(|err| {
        tracing::error!(
            "Error confirming payment for order {}: {}",
            payload.order_id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn update_order_status<'e, E: PgExecutor<'e>>(
    e: E,
    order_id: String,
    new_status: OrderStatus,
) -> Result<bool, Error> {
    sqlx::query!(
        r#"
        WITH valid_transition AS (
            SELECT CASE
                WHEN orders.status = 'AWAITING_ACKNOWLEDGEMENT' AND $2 = 'PREPARING' THEN TRUE
                WHEN orders.status = 'PREPARING' AND $2 = 'IN_TRANSIT' THEN TRUE
                WHEN orders.status = 'IN_TRANSIT' AND $2 = 'DELIVERED' THEN TRUE
                ELSE FALSE
            END AS is_valid
            FROM orders
            WHERE id = $1
        ),
        updated_order AS (
            UPDATE orders
            SET status = $2
            WHERE id = $1
              AND (SELECT is_valid FROM valid_transition)
            RETURNING id
        )
        SELECT EXISTS(SELECT 1 FROM updated_order);
        "#,
        order_id,
        new_status.to_string()
    )
    .fetch_optional(e)
    .await
    .map(|opt| opt.is_some())
    .map_err(|err| {
        tracing::error!("Error updating status for order {}: {}", order_id, err);
        Error::UnexpectedError
    })
}

// pub async fn update_order_item_status(
//     e: E,
//     order_item_id: String,
//     new_status: OrderStatus,
// ) -> Result<bool, Error> {
//     sqlx::query!(
//         r#"
//         WITH valid_transition AS (
//             SELECT CASE
//                 WHEN order_items.status = 'AWAITING_ACKNOWLEDGEMENT' AND $2 = 'PREPARING' THEN TRUE
//                 WHEN order_items.status = 'PREPARING' AND $2 = 'IN_TRANSIT' THEN TRUE
//                 WHEN order_items.status = 'IN_TRANSIT' AND $2 = 'DELIVERED' THEN TRUE
//                 ELSE FALSE
//             END AS is_valid
//             FROM order_items
//             WHERE id = $1
//         ),
//         updated_item AS (
//             UPDATE order_items
//             SET status = $2
//             WHERE id = $1
//               AND (SELECT is_valid FROM valid_transition)
//             RETURNING id
//         )
//         SELECT EXISTS(SELECT 1 FROM updated_item);
//         "#,
//         order_item_id,
//         new_status.to_string()
//     )
//     .fetch_optional(e)
//     .await
//     .map(|opt| opt.is_some())
//     .map_err(|err| {
//         tracing::error!(
//             "Error updating status for order item {}: {}",
//             order_item_id,
//             err
//         );
//         Error::UnexpectedError
//     })
// }

pub fn is_owner(order: &Order, user: &User) -> bool {
    order.owner_id == user.id
}
