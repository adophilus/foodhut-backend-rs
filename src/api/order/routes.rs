use std::sync::Arc;

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
                &ctx.db_conn.pool,
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
                &ctx.db_conn.pool,
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
        true => repository::order::find_full_order_by_id(&ctx.db_conn.pool, id).await,
        false => {
            repository::order::find_full_order_by_id_and_owner_id(
                &ctx.db_conn.pool,
                id,
                auth.user.id,
            )
            .await
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
    pub as_kitchen: Option<bool>, // Optional parameter to signify if the request is made as a kitchen
}

// async fn update_order_item_status(
//     Path((order_id, order_item_id)): Path<(String, String)>,
//     State(ctx): State<Arc<Context>>,
//     auth: Auth,
//     Json(payload): Json<UpdateOrderItemPayload>,
// ) -> impl IntoResponse {
//     // Fetch the current order item to determine its status
//     let current_order_item =
//         match repository::order::find_order_item_by_id(&ctx.db_conn.pool, order_item_id.clone())
//             .await
//         {
//             Ok(Some(item)) => item,
//             Ok(None) => {
//                 return (
//                     StatusCode::NOT_FOUND,
//                     Json(json!({ "message": "Order item not found" })),
//                 );
//             }
//             Err(_) => {
//                 return (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(json!({ "message": "Failed to retrieve order item" })),
//                 );
//             }
//         };
//
//     let as_kitchen = payload.as_kitchen.unwrap_or(false);
//
//     if as_kitchen {
//         // Check if the user owns the kitchen
//         match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id.clone()).await
//         {
//             Ok(Some(kitchen)) => {
//                 // Ensure that the kitchen ID matches the order item's kitchen_id
//                 if kitchen.id != current_order_item.kitchen_id {
//                     return (
//                         StatusCode::FORBIDDEN,
//                         Json(json!({ "message": "Kitchen does not own this order item" })),
//                     );
//                 }
//
//                 // Ensure the kitchen is allowed to update the status (kitchen status transitions)
//                 match (current_order_item.status, payload.status.clone()) {
//                     (
//                         repository::order::OrderStatus::AwaitingAcknowledgement,
//                         repository::order::OrderStatus::Preparing,
//                     )
//                     | (
//                         repository::order::OrderStatus::Preparing,
//                         repository::order::OrderStatus::InTransit,
//                     ) => {
//                         // Update order item status as kitchen
//                         if repository::order::update_order_item_status(
//                             ctx.db_conn.clone(),
//                             order_item_id.clone(),
//                             payload.status.clone(),
//                         )
//                         .await
//                         .unwrap_or(false)
//                         {
//                             return (
//                                 StatusCode::OK,
//                                 Json(
//                                     json!({ "message": "Order item status updated successfully" }),
//                                 ),
//                             );
//                         }
//                     }
//                     _ => {
//                         return (
//                             StatusCode::BAD_REQUEST,
//                             Json(json!({ "message": "Invalid status transition for kitchen" })),
//                         );
//                     }
//                 }
//             }
//             Ok(None) => {
//                 return (
//                     StatusCode::FORBIDDEN,
//                     Json(json!({ "message": "User does not own a kitchen" })),
//                 );
//             }
//             Err(_) => {
//                 return (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(json!({ "message": "Failed to retrieve kitchen" })),
//                 );
//             }
//         }
//     } else {
//         // For users (non-kitchen), ensure that the user owns the order item
//         if current_order_item.owner_id != auth.user.id {
//             return (
//                 StatusCode::FORBIDDEN,
//                 Json(json!({ "message": "User does not own this order item" })),
//             );
//         }
//
//         // For users, ensure valid transitions (user status transitions)
//         match (current_order_item.status, payload.status.clone()) {
//             (
//                 repository::order::OrderStatus::InTransit,
//                 repository::order::OrderStatus::Delivered,
//             ) => {
//                 // Update order item status as user
//                 if repository::order::update_order_item_status(
//                     ctx.db_conn.clone(),
//                     order_item_id.clone(),
//                     payload.status.clone(),
//                 )
//                 .await
//                 .unwrap_or(false)
//                 {
//                     return (
//                         StatusCode::OK,
//                         Json(json!({ "message": "Order item status updated successfully" })),
//                     );
//                 }
//             }
//             _ => {
//                 return (
//                     StatusCode::BAD_REQUEST,
//                     Json(json!({ "message": "Invalid status transition for user" })),
//                 );
//             }
//         }
//     }
//
//     (
//         StatusCode::INTERNAL_SERVER_ERROR,
//         Json(json!({ "message": "Failed to update order item status" })),
//     )
// }

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
    let order = match repository::order::find_by_id(&ctx.db_conn.pool, id).await {
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

    let mut tx = match ctx.db_conn.pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to start transaction" })),
            );
        }
    };

    let details = match utils::payment::initialize_payment_for_order(
        ctx.clone(),
        &mut tx,
        utils::payment::InitializePaymentForOrder {
            method,
            order,
            payer: auth.user.clone(),
        },
    )
    .await
    {
        Ok(details) => details,
        Err(utils::payment::Error::AlreadyPaid) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Payment has already been made" })),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Payment failed!" })),
            )
        }
    };

    match tx.commit().await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Sorry an error occurred" })),
            );
        }
    };

    (StatusCode::OK, Json(json!(details)))
}

pub fn get_router() -> Router<Arc<Context>> {
    // TODO: add endpoint for manually verifying online payment
    Router::new()
        .route("/", get(get_orders))
        .route("/:id", get(get_order_by_id))
        // .route(
        //     "/:order_id/items/:order_item_id/status",
        //     put(update_order_item_status),
        // )
        .route("/:id/pay", post(pay_for_order))
}
