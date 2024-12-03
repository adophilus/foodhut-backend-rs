use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use sqlx::PgExecutor;
use std::{convert::Into, str::FromStr};
use ulid::Ulid;

use crate::{
    define_paginated,
    modules::{cart::repository::FullCartItem, kitchen::repository::Kitchen, meal::repository::Meal},
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
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
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
        .fold(BigDecimal::from(0), |acc, item| acc + item.meal.price);
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
        WITH parsed_order_items AS (
            SELECT
                orders.id AS order_id,
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
                json_array_elements(orders.items) AS item -- Expand JSON array into individual rows
            FROM orders
            WHERE
                orders.id = $1 -- Filter only by order_id
        ),
        expanded_items AS (
            SELECT
                parsed_order_items.*,
                (item->>'price')::NUMERIC AS item_price,
                (item->>'quantity')::INT AS item_quantity,
                item->>'meal_id' AS item_meal_id
            FROM parsed_order_items
        ),
        joined_meals AS (
            SELECT
                expanded_items.*,
                meals.id AS meal_id,
                meals.name AS meal_name,
                meals.description,
                meals.rating,
                meals.price AS meal_price,
                meals.original_price, -- Include original_price
                meals.likes,
                meals.cover_image,
                meals.is_available,
                meals.kitchen_id AS meal_kitchen_id,
                meals.created_at AS meal_created_at,
                meals.updated_at AS meal_updated_at
            FROM expanded_items
            LEFT JOIN meals ON expanded_items.item_meal_id = meals.id
        ),
        grouped_orders AS (
            SELECT
                order_id as id,
                status,
                payment_method,
                delivery_fee,
                service_fee,
                sub_total,
                total,
                delivery_address,
                delivery_date,
                dispatch_rider_note,
                kitchen_id,
                owner_id,
                created_at,
                updated_at,
                COALESCE(
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT(
                            'price', item_price,
                            'quantity', item_quantity,
                            'meal_id', item_meal_id,
                            'meal', JSONB_BUILD_OBJECT(
                                'id', meal_id,
                                'name', meal_name,
                                'description', description,
                                'rating', rating,
                                'original_price', original_price, -- Map original_price
                                'price', meal_price,
                                'likes', likes,
                                'cover_image', cover_image,
                                'is_available', is_available,
                                'kitchen_id', meal_kitchen_id,
                                'created_at', meal_created_at,
                                'updated_at', meal_updated_at
                            )
                        )
                    ),
                    '[]'::JSONB
                ) AS items
            FROM joined_meals
            GROUP BY
                order_id, status, payment_method, delivery_fee, service_fee,
                sub_total, total, delivery_address, delivery_date, dispatch_rider_note,
                kitchen_id, owner_id, created_at, updated_at
        )
        SELECT * FROM grouped_orders;
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
        WITH parsed_order_items AS (
            SELECT
                orders.id AS order_id,
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
                json_array_elements(orders.items) AS item -- Expand JSON array into individual rows
            FROM orders
            WHERE
                orders.id = $1
                AND orders.owner_id = $2
        ),
        expanded_items AS (
            SELECT
                parsed_order_items.*,
                (item->>'price')::NUMERIC AS item_price,
                (item->>'quantity')::INT AS item_quantity,
                item->>'meal_id' AS item_meal_id
            FROM parsed_order_items
        ),
        joined_meals AS (
            SELECT
                expanded_items.*,
                meals.id AS meal_id,
                meals.name AS meal_name,
                meals.description,
                meals.rating,
                meals.price AS meal_price,
                meals.original_price, -- Include original_price
                meals.likes,
                meals.cover_image,
                meals.is_available,
                meals.kitchen_id AS meal_kitchen_id,
                meals.created_at AS meal_created_at,
                meals.updated_at AS meal_updated_at
            FROM expanded_items
            LEFT JOIN meals ON expanded_items.item_meal_id = meals.id
        ),
        grouped_orders AS (
            SELECT
                order_id as id,
                status,
                payment_method,
                delivery_fee,
                service_fee,
                sub_total,
                total,
                delivery_address,
                delivery_date,
                dispatch_rider_note,
                kitchen_id,
                owner_id,
                created_at,
                updated_at,
                COALESCE(
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT(
                            'price', item_price,
                            'quantity', item_quantity,
                            'meal_id', item_meal_id,
                            'meal', JSONB_BUILD_OBJECT(
                                'id', meal_id,
                                'name', meal_name,
                                'description', description,
                                'rating', rating,
                                'original_price', original_price, -- Map original_price
                                'price', meal_price,
                                'likes', likes,
                                'cover_image', cover_image,
                                'is_available', is_available,
                                'kitchen_id', meal_kitchen_id,
                                'created_at', meal_created_at,
                                'updated_at', meal_updated_at
                            )
                        )
                    ),
                    '[]'::JSONB
                ) AS items
            FROM joined_meals
            GROUP BY
                order_id, status, payment_method, delivery_fee, service_fee,
                sub_total, total, delivery_address, delivery_date, dispatch_rider_note,
                kitchen_id, owner_id, created_at, updated_at
        )
        SELECT * FROM grouped_orders;
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

// pub async fn find_full_order_by_id_and_owner_id(
//     e: E,
//     order_id: String,
//     owner_id: String,
// ) -> Result<Option<FullOrder>, Error> {
//     sqlx::query_as!(
//         FullOrder,
//         r#"
//         WITH order_data AS (
//             SELECT orders.*,
//                    COALESCE(
//                        JSONB_AGG(
//                            JSONB_BUILD_OBJECT(
//                                'id', full_order_items.item_id,
//                                'status', full_order_items.item_status,
//                                'price', full_order_items.price,
//                                'meal_id', full_order_items.meal_id,
//                                'order_id', full_order_items.order_id,
//                                'kitchen_id', full_order_items.kitchen_id,
//                                'owner_id', full_order_items.owner_id,
//                                'created_at', full_order_items.item_created_at,
//                                'updated_at', full_order_items.item_updated_at,
//                                'meal', JSONB_BUILD_OBJECT(
//                                    'id', full_order_items.meal_id,
//                                    'name', full_order_items.meal_name,
//                                    'description', full_order_items.description,
//                                    'rating', full_order_items.rating,
//                                    'price', full_order_items.meal_price,
//                                    'likes', full_order_items.likes,
//                                    'cover_image', full_order_items.cover_image,
//                                    'is_available', full_order_items.is_available,
//                                    'kitchen_id', full_order_items.meal_kitchen_id,
//                                    'created_at', full_order_items.meal_created_at,
//                                    'updated_at', full_order_items.meal_updated_at
//                                )
//                            )
//                        ) FILTER (WHERE full_order_items.item_id IS NOT NULL),
//                        '[]'::jsonb
//                    ) AS items
//             FROM orders
//             LEFT JOIN (
//                 SELECT order_items.id AS item_id,
//                        order_items.order_id,
//                        order_items.kitchen_id,
//                        order_items.price,
//                        order_items.status AS item_status,
//                        order_items.created_at AS item_created_at,
//                        order_items.updated_at AS item_updated_at,
//                        order_items.owner_id,
//                        meals.id AS meal_id,
//                        meals.name AS meal_name,
//                        meals.description,
//                        meals.rating,
//                        meals.price AS meal_price,
//                        meals.likes,
//                        meals.cover_image,
//                        meals.is_available,
//                        meals.kitchen_id AS meal_kitchen_id,
//                        meals.created_at AS meal_created_at,
//                        meals.updated_at AS meal_updated_at
//                 FROM order_items
//                 LEFT JOIN meals ON order_items.meal_id = meals.id
//             ) AS full_order_items ON orders.id = full_order_items.order_id
//             WHERE orders.id = $1 AND orders.owner_id = $2
//             GROUP BY orders.id
//         )
//         SELECT * FROM order_data;
//         "#,
//         order_id,
//         owner_id
//     )
//     .fetch_optional(e)
//     .await
//     .map_err(|err| {
//         tracing::error!(
//             "Error occurred while trying to fetch full order by id {} and owner id {}: {}",
//             order_id,
//             owner_id,
//             err
//         );
//         Error::UnexpectedError
//     })
// }

#[derive(Clone, Debug, Deserialize)]
pub struct Filters {
    pub owner_id: Option<String>,
    pub status: Option<OrderSimpleStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub kitchen_id: Option<String>,
}

pub async fn find_many<'e, E: PgExecutor<'e>>(
    e: E,
    pagination: Pagination,
    filters: Filters,
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
            LIMIT $1 OFFSET $2
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
                kitchens AS kitchen
            FROM
                filtered_orders
            LEFT JOIN
                meals
            ON meals.id = filtered_orders.item->>'meal_id'
            LEFT JOIN
                kitchens
            ON kitchens.id = filtered_orders.kitchen_id
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
            JSON_AGG(query_result) AS items,
            JSONB_BUILD_OBJECT(
                'total', total_rows,
                'per_page', $1,
                'page', $2 / $1 + 1
            ) AS meta
        FROM
            query_result,
            total_count
        GROUP BY
            total_count.total_rows
        "#,
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
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

pub async fn confirm_payment<'e, E: PgExecutor<'e>>(e: E, order_id: String) -> Result<bool, Error> {
    sqlx::query!(
        r#"
        UPDATE orders SET status = 'AWAITING_ACKNOWLEDGEMENT'
        WHERE id = $1
          AND status = 'AWAITING_PAYMENT'
        "#,
        order_id
    )
    .fetch_optional(e)
    .await
    .map(|opt| opt.is_some())
    .map_err(|err| {
        tracing::error!("Error confirming payment for order {}: {}", order_id, err);
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
