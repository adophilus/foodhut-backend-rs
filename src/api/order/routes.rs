use std::{borrow::Cow, ops::Deref, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tower::util::error::optional::None;
use validator::{Validate, ValidationError};

use crate::{
    api::auth::middleware::Auth,
    repository,
    types::Context,
    utils::{self, pagination::Pagination},
};

#[derive(Deserialize)]
struct Filters {
    status: Option<repository::order::OrderSimpleStatus>,
    kitchen_id: Option<String>,
}

async fn get_orders(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    pagination: Pagination,
    Query(filters): Query<Filters>,
) -> impl IntoResponse {
    let orders = match repository::user::is_admin(&auth.user) {
        true => {
            repository::order::find_many(
                ctx.db_conn.clone(),
                pagination.clone(),
                repository::order::Filters {
                    owner_id: None,
                    payment_method: None,
                    status: filters.status,
                    kitchen_id: filters.kitchen_id,
                },
            )
            .await
        }
        false => {
            repository::order::find_many(
                ctx.db_conn.clone(),
                pagination.clone(),
                repository::order::Filters {
                    owner_id: Some(auth.user.id.clone()),
                    payment_method: None,
                    status: filters.status,
                    kitchen_id: filters.kitchen_id,
                },
            )
            .await
        }
    };

    match orders {
        Ok(paginated_orders) => (StatusCode::OK, Json(json!(paginated_orders))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch orders"})),
        ),
    }
}

async fn get_order_by_id(
    Path(id): Path<String>,
    auth: Auth,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    let maybe_order = match repository::user::is_admin(&auth.user) {
        true => repository::order::find_by_id(ctx.db_conn.clone(), id).await,
        false => {
            repository::order::find_by_id_and_owner_id(ctx.db_conn.clone(), id, auth.user.id).await
        }
    };
    match maybe_order {
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

#[derive(Deserialize)]
pub struct UpdateOrderItemPayload {
    pub status: repository::order::OrderStatus,
}

async fn update_order_item_status(
    Path((order_id, order_item_id)): Path<(String, String)>,
    State(ctx): State<Arc<Context>>,
    Query(kitchen_id): Query<Option<String>>,
    auth: Auth,
    Json(payload): Json<UpdateOrderItemPayload>,
) -> impl IntoResponse {
    if let Some(kitchen_id) = kitchen_id {
        match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
            Ok(Some(kitchen)) if kitchen.id == kitchen_id => {
                if repository::order::update_order_item_status(
                    ctx.db_conn.clone(),
                    order_item_id,
                    payload.status,
                )
                .await
                .unwrap_or(false)
                {
                    return (
                        StatusCode::OK,
                        Json(json!({ "message": "Order item status updated successfully" })),
                    );
                }
            }
            Ok(_) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "message": "User does not own this kitchen" })),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to verify kitchen ownership" })),
                );
            }
        }
    } else {
        if repository::order::update_order_item_status(
            ctx.db_conn.clone(),
            order_item_id,
            payload.status,
        )
        .await
        .unwrap_or(false)
        {
            return (
                StatusCode::OK,
                Json(json!({ "message": "Order item status updated successfully" })),
            );
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Failed to update order item status" })),
    )
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
        Ok(details) => (StatusCode::OK, Json(json!(details))),
        Err(utils::payment::Error::AlreadyPaid) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Payment has already been made" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Payment failed!" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    // TODO: add endpoint for manually verifying online payment
    Router::new()
        .route("/", get(get_orders))
        .route(
            "/:order_id/items/:order_item_id/status",
            get(get_order_by_id).patch(update_order_item_status),
        )
        .route("/:id/pay", post(pay_for_order))
}
