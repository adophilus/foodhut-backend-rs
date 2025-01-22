use std::sync::Arc;

use super::service;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::{
    modules::{auth::middleware::Auth, kitchen, notification, payment, user},
    types::Context,
    utils::pagination::Pagination,
};

use super::repository::{self, OrderSimpleStatus};

#[derive(Deserialize)]
struct Filters {
    status: Option<OrderSimpleStatus>,
    kitchen_id: Option<String>,
}

async fn get_orders(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    pagination: Pagination,
    Query(filters): Query<Filters>,
) -> impl IntoResponse {
    let orders = match user::repository::is_admin(&auth.user) {
        true => {
            repository::find_many_as_admin(
                &ctx.db_conn.pool,
                pagination.clone(),
                repository::FindManyAsAdminFilters {
                    owner_id: None,
                    payment_method: None,
                    status: filters.status,
                    kitchen_id: filters.kitchen_id,
                },
            )
            .await
        }
        false => {
            if filters.kitchen_id.is_some() {
                repository::find_many_as_kitchen(
                    &ctx.db_conn.pool,
                    pagination.clone(),
                    repository::FindManyAsKitchenFilters {
                        owner_id: Some(auth.user.id.clone()),
                        payment_method: None,
                        status: filters.status,
                        kitchen_id: filters.kitchen_id,
                    },
                )
                .await
            } else {
                repository::find_many_as_user(
                    &ctx.db_conn.pool,
                    pagination.clone(),
                    repository::FindManyAsUserFilters {
                        owner_id: Some(auth.user.id.clone()),
                        payment_method: None,
                        status: filters.status,
                        kitchen_id: filters.kitchen_id,
                    },
                )
                .await
            }
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
    if user::repository::is_admin(&auth.user) {
        match repository::find_full_order_by_id(&ctx.db_conn.pool, id).await {
            Ok(Some(order)) => {
                let owner =
                    match user::repository::find_by_id(&ctx.db_conn.pool, order.owner_id.clone())
                        .await
                    {
                        Ok(Some(owner)) => owner,
                        _ => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "error": "Failed to fetch order owner" })),
                            )
                        }
                    };

                (StatusCode::OK, Json(json!(order.add_owner(owner))))
            }
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Order not found" })),
            ),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch orders"})),
            ),
        }
    } else {
        match repository::find_full_order_by_id_and_owner_id(&ctx.db_conn.pool, id, auth.user.id)
            .await
        {
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
}

#[derive(Deserialize)]
pub struct UpdateOrderStatusPayload {
    pub status: repository::OrderStatus,
    pub as_kitchen: Option<bool>, // Optional parameter to signify if the request is made as a kitchen
}

async fn update_order_status(
    Path(order_id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<UpdateOrderStatusPayload>,
) -> impl IntoResponse {
    // Fetch the current order to determine its status
    let order = match repository::find_by_id(&ctx.db_conn.pool, order_id.clone()).await {
        Ok(Some(order)) => order,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "message": "Order not found" })),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "message": "Failed to retrieve order" })),
            );
        }
    };

    let mut tx = match ctx.db_conn.clone().pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start database transaction: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Sorry, an error occurred" })),
            );
        }
    };

    let as_kitchen = payload.as_kitchen.unwrap_or(false);

    if as_kitchen {
        // Check if the user owns the kitchen
        match kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone()).await {
            Ok(Some(kitchen)) => {
                // Ensure that the kitchen ID matches the order item's kitchen_id
                if kitchen.id != order.kitchen_id {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(json!({ "message": "Kitchen does not own this order" })),
                    );
                }

                // Ensure the kitchen is allowed to update the status (kitchen status transitions)
                match (order.status, payload.status.clone()) {
                    (
                        repository::OrderStatus::AwaitingAcknowledgement,
                        repository::OrderStatus::Preparing,
                    )
                    | (repository::OrderStatus::Preparing, repository::OrderStatus::InTransit) => {
                        // Update order item status as kitchen
                        match repository::update_order_status(
                            &mut *tx,
                            order.id.clone(),
                            payload.status.clone(),
                        )
                        .await
                        {
                            Ok(Some(order)) => {
                                let order_owner = match user::repository::find_by_id(
                                    &mut *tx,
                                    order.owner_id.clone(),
                                )
                                .await
                                {
                                    Ok(Some(user)) => user,
                                    Ok(None) => {
                                        return (
                                            StatusCode::NOT_FOUND,
                                            Json(json!({ "error": "User not found" })),
                                        )
                                    }
                                    _ => {
                                        return (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            Json(
                                                json!({ "message": "Failed to update order status" }),
                                            ),
                                        )
                                    }
                                };

                                if let Err(err) = tx.commit().await {
                                    tracing::error!(
                                        "Failed to commit database transaction: {}",
                                        err
                                    );
                                    return (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(json!({ "message": "Failed to update order status" })),
                                    );
                                }

                                notification::service::send(
                                    ctx,
                                    notification::service::Notification::order_status_updated(
                                        order,
                                        order_owner,
                                    ),
                                    notification::service::Backend::Push,
                                )
                                .await;

                                (
                                    StatusCode::OK,
                                    Json(json!({ "message": "Order status updated successfully" })),
                                )
                            }
                            Ok(None) => (
                                StatusCode::NOT_FOUND,
                                Json(json!({ "error": "Order not found" })),
                            ),
                            _ => (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "message": "Failed to update order status" })),
                            ),
                        }
                    }
                    _ => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "message": "Invalid status transition for kitchen" })),
                        );
                    }
                }
            }
            Ok(None) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "message": "User does not own a kitchen" })),
                );
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to retrieve kitchen" })),
                );
            }
        }
    } else {
        // For users (non-kitchen), ensure that the user owns the order
        if !repository::is_owner(&order, &auth.user) {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "User does not own this order item" })),
            );
        }

        // For users, ensure valid transitions (user status transitions)
        match (order.status.clone(), payload.status.clone()) {
            (repository::OrderStatus::InTransit, repository::OrderStatus::Delivered) => {
                // Update order status as user
                match service::mark_order_as_delivered(
                    ctx.clone(),
                    &mut tx,
                    service::MarkOrderAsDeliveredPayload {
                        order: order.clone(),
                    },
                )
                .await
                {
                    Ok(_) => {
                        let kithen_owner = match user::repository::find_by_kitchen_id(
                            &mut *tx,
                            order.kitchen_id.clone(),
                        )
                        .await
                        {
                            Ok(Some(user)) => user,
                            Ok(None) => {
                                return (
                                    StatusCode::NOT_FOUND,
                                    Json(json!({ "error": "User not found" })),
                                )
                            }
                            _ => {
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({ "message": "Failed to update order status" })),
                                )
                            }
                        };

                        notification::service::send(
                            ctx,
                            notification::service::Notification::order_status_updated(
                                order,
                                kithen_owner,
                            ),
                            notification::service::Backend::Push,
                        )
                        .await;

                        if let Err(err) = tx.commit().await {
                            tracing::error!("Failed to commit database transaction: {}", err);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "message": "Failed to update order status" })),
                            );
                        }

                        (
                            StatusCode::OK,
                            Json(json!({ "message": "Order status updated successfully" })),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "message": "Failed to update order status" })),
                    ),
                }
            }
            _ => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "message": "Invalid status transition for user" })),
            ),
        }
    }
}

#[derive(Deserialize)]
struct PayForOrderPayload {
    with: repository::PaymentMethod,
}

async fn pay_for_order(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
    Json(payload): Json<PayForOrderPayload>,
) -> impl IntoResponse {
    let order = match repository::find_by_id(&ctx.db_conn.pool, id).await {
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

    match service::pay_for_order(
        ctx.clone(),
        service::PayForOrderPayload {
            payment_method: payload.with,
            order,
            payer: auth.user,
        },
    )
    .await
    {
        Ok(details) => (StatusCode::OK, Json(json!(details))),
        Err(service::PayForOrderError::AlreadyPaid) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Payment has already been made" })),
            )
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Payment failed!" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    // TODO: add endpoint for manually verifying online payment
    Router::new()
        .route("/", get(get_orders))
        .route("/:id", get(get_order_by_id))
        .route("/:order_id/status", put(update_order_status))
        .route("/:id/pay", post(pay_for_order))
}
