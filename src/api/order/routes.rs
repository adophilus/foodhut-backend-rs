use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationError};

use crate::{
    api::auth::middleware::Auth,
    repository,
    types::Context,
    utils::{self, pagination::Pagination},
};

const order_TYPES: [&str; 4] = ["Chinese", "Cuisine", "Fast Food", "Local"];

#[derive(Deserialize, Validate)]
struct CreateOrderPayload {
    pub name: String,
    pub address: String,
    pub phone_number: String,
    #[validate(custom(function = "validate_order_type"))]
    #[serde(rename = "type")]
    pub type_: String,
    #[validate(custom(function = "validate_opening_time"))]
    pub opening_time: String,
    #[validate(custom(function = "validate_closing_time"))]
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
}

fn validate_order_type(type_: &str) -> Result<(), ValidationError> {
    match order_TYPES.contains(&type_) {
        true => Ok(()),
        false => Err(ValidationError::new("INVALID_order_TYPE")
            .with_message(Cow::from("Invalid order type"))),
    }
}

fn validate_opening_time(time_str: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid opening time regex");
    match regex.is_match(time_str) {
        true => Ok(()),
        false => Err(
            ValidationError::new("INVALID_OPENING_TIME").with_message(Cow::from(
                r"Opening time must be in 24 hour format (e.g: 08:00)",
            )),
        ),
    }
}

fn validate_closing_time(time_str: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid closing time regex");
    match regex.is_match(time_str) {
        true => Ok(()),
        false => Err(
            ValidationError::new("INVALID_CLOSING_TIME").with_message(Cow::from(
                r"Closing time must be in 24 hour format (e.g: 20:00)",
            )),
        ),
    }
}

async fn create_order(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateOrderPayload>,
) -> impl IntoResponse {
    if let Err(errors) = payload.validate() {
        return utils::validation::into_response(errors);
    }

    match repository::order::create(
        ctx.db_conn.clone(),
        repository::order::CreateOrderPayload {
            name: payload.name,
            address: payload.address,
            type_: payload.type_,
            phone_number: payload.phone_number,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            owner_id: auth.user.id,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Order created!"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Order creation failed"})),
        ),
    }
}

async fn get_orders(
    State(ctx): State<Arc<Context>>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::order::find_many(ctx.db_conn.clone(), pagination.clone()).await {
        Ok(paginated_orders) => (StatusCode::OK, Json(json!(paginated_orders))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch orders"})),
        ),
    }
}

async fn get_order_by_profile(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::order::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
        Ok(Some(order)) => (StatusCode::OK, Json(json!(order))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Order not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch order"})),
        ),
    }
}

async fn get_order_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::order::find_by_id(ctx.db_conn.clone(), id).await {
        Ok(Some(order)) => (StatusCode::OK, Json(json!(order))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Order not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch orders"})),
        ),
    }
}

async fn fetch_order_types() -> impl IntoResponse {
    Json(json!(order_TYPES))
}

#[derive(Deserialize, Validate)]
pub struct UpdateOrderPayload {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone_number: Option<String>,
    #[validate(custom(function = "validate_order_type"))]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[validate(custom(function = "validate_opening_time"))]
    pub opening_time: Option<String>,
    #[validate(custom(function = "validate_closing_time"))]
    pub closing_time: Option<String>,
    pub preparation_time: Option<String>,
    pub delivery_time: Option<String>,
}

async fn update_order_by_profile(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<UpdateOrderPayload>,
) -> Response {
    match repository::order::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
        Ok(Some(order)) => {
            update_order_by_id(Path { 0: order.id }, State(ctx), Json(payload))
                .await
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Order not found" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch order" })),
        )
            .into_response(),
    }
}

async fn update_order_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<UpdateOrderPayload>,
) -> impl IntoResponse {
    match repository::order::update_by_id(
        ctx.db_conn.clone(),
        id,
        repository::order::UpdateOrderPayload {
            name: payload.name,
            address: payload.address,
            phone_number: payload.phone_number,
            type_: payload.type_,
            opening_time: payload.opening_time,
            closing_time: payload.closing_time,
            preparation_time: payload.preparation_time,
            delivery_time: payload.delivery_time,
            rating: None,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Order updated successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to update order" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_order).get(get_orders))
        .route(
            "/profile",
            get(get_order_by_profile).patch(update_order_by_profile),
        )
        .route("/:id", get(get_order_by_id).patch(update_order_by_id))
        .route("/types", get(fetch_order_types))
}
