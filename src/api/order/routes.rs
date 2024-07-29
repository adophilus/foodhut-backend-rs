use std::{borrow::Cow, ops::Deref, sync::Arc};

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

async fn get_orders(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::order::find_many_by_owner_id(
        ctx.db_conn.clone(),
        auth.user.id.clone(),
        pagination.clone(),
    )
    .await
    {
        Ok(paginated_orders) => (StatusCode::OK, Json(json!(paginated_orders))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch orders"})),
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

#[derive(Deserialize, Validate)]
pub struct UpdateOrderPayload {
    pub status: repository::order::OrderStatus,
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
            status: payload.status,
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

#[derive(Deserialize)]
struct PayForOrderPayload {
    with: repository::order::PaymentMethod,
}

async fn pay_for_order(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
    Json(payload): Json<PayForOrderPayload>,
) -> impl IntoResponse {
    let order = match repository::order::find_by_id(ctx.db_conn.clone(), id).await {
        Ok(Some(order)) => order,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Order not found" })),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to find order by id"})),
            )
        }
    };

    let method = match payload.with {
        repository::order::PaymentMethod::Online => utils::payment::PaymentMethod::Online,
        repository::order::PaymentMethod::Wallet => utils::payment::PaymentMethod::Wallet,
    };

    match utils::payment::initialize_payment_for_order(
        ctx.clone(),
        utils::payment::InitializePaymentForOrder {
            method,
            order,
            payer: auth.user.clone(),
        },
    )
    .await
    {
        // TODO: this payment detais should actually be presented to the user
        Ok(details) => (
            StatusCode::OK,
            Json(json!({ "message": "Payment successful" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Payment failed!" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(get_orders))
        .route("/:id", get(get_order_by_id).patch(update_order_by_id))
        .route("/:id/pay", post(pay_for_order))
}
