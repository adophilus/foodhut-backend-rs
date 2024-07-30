use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationError};

use crate::{
    api::auth::middleware::Auth,
    repository::{
        self,
        cart::{CartItem, CartItems},
    },
    types::Context,
    utils::{self, pagination::Pagination},
};

async fn create_cart(State(ctx): State<Arc<Context>>, auth: Auth) -> impl IntoResponse {
    match repository::cart::create(
        ctx.db_conn.clone(),
        repository::cart::CreateCartPayload {
            owner_id: auth.user.id,
        },
    )
    .await
    {
        Ok(cart) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Cart created!", "id": cart.id })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Cart creation failed"})),
        ),
    }
}

async fn get_carts(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::cart::find_many_by_owner_id(
        ctx.db_conn.clone(),
        pagination.clone(),
        auth.user.id,
    )
    .await
    {
        Ok(paginated_carts) => (StatusCode::OK, Json(json!(paginated_carts))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch carts"})),
        ),
    }
}

async fn get_active_cart(State(ctx): State<Arc<Context>>, auth: Auth) -> impl IntoResponse {
    match repository::cart::find_active_by_owner_id(ctx.db_conn.clone(), auth.user.id.clone()).await
    {
        Ok(Some(cart)) => (StatusCode::OK, Json(json!(cart))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No active cart found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch active cart" })),
        ),
    }
}

async fn get_cart_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    let cart = match repository::cart::find_by_id(ctx.db_conn.clone(), id.clone()).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to find cart"})),
            )
        }
        Ok(Some(cart)) => cart,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Cart not found"})),
            )
        }
    };

    if !repository::cart::is_owner(auth.user.clone(), cart.clone()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this cart"})),
        );
    }

    (StatusCode::OK, Json(json!(cart)))
}

#[derive(Deserialize, Validate)]
pub struct SetMealInCartPayload {
    quantity: i32,
}

async fn set_meal_in_active_cart(
    Path(meal_id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<SetMealInCartPayload>,
) -> impl IntoResponse {
    let cart =
        match repository::cart::find_active_by_owner_id(ctx.db_conn.clone(), auth.user.id.clone())
            .await
        {
            Ok(None) => match repository::cart::create(
                ctx.db_conn.clone(),
                repository::cart::CreateCartPayload {
                    owner_id: auth.user.id.clone(),
                },
            )
            .await
            {
                Ok(cart) => cart,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to set item in cart"})),
                    )
                }
            },
            Ok(Some(cart)) => cart,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to set item in cart"})),
                )
            }
        };

    let meal = match repository::meal::find_by_id(ctx.db_conn.clone(), meal_id).await {
        Ok(Some(meal)) => meal,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Meal not found" })),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch meal" })),
            )
        }
    };

    let mut found = false;
    let mut index = 0;

    let mut items = cart.items.0;
    for item in items.iter_mut() {
        if item.meal_id == meal.id.clone() {
            found = true;
            item.quantity = payload.quantity;
            break;
        }
        index += 1;
    }

    if !found {
        items.push(CartItem {
            meal_id: meal.id.clone(),
            quantity: payload.quantity,
        });
    } else {
        if payload.quantity == 0 {
            items.remove(index);
        }
    }

    match repository::cart::update_by_id(
        ctx.db_conn.clone(),
        cart.id.clone(),
        repository::cart::UpdateCartPayload {
            items: Some(CartItems(items)),
            status: None,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Cart updated successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to update cart" })),
        ),
    }
}

async fn remove_meal_from_active_cart(
    Path(meal_id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
) -> impl IntoResponse {
    let cart =
        match repository::cart::find_active_by_owner_id(ctx.db_conn.clone(), auth.user.id.clone())
            .await
        {
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to find cart"})),
                )
            }
            Ok(Some(cart)) => cart,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Cart not found"})),
                )
            }
        };

    let meal = match repository::meal::find_by_id(ctx.db_conn.clone(), meal_id).await {
        Ok(Some(meal)) => meal,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Meal not found" })),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to fetch meal" })),
            )
        }
    };

    let mut found = false;

    let new_items = cart
        .items
        .0
        .into_iter()
        .filter(|item| {
            if item.meal_id == meal.id {
                found = true;
                return false;
            }

            true
        })
        .collect::<Vec<_>>();

    if !found {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Meal not found in cart" })),
        );
    }

    match repository::cart::update_by_id(
        ctx.db_conn.clone(),
        cart.id.clone(),
        repository::cart::UpdateCartPayload {
            items: Some(CartItems(new_items)),
            status: None,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Meal removed from cart successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to remove meal from cart" })),
        ),
    }
}

#[derive(Deserialize)]
pub struct CheckoutCartByIdPayload {
    payment_method: repository::order::PaymentMethod,
    delivery_address: String,
    dispatch_rider_note: String,
}

pub async fn checkout_active_cart(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CheckoutCartByIdPayload>,
) -> impl IntoResponse {
    let cart =
        match repository::cart::find_active_by_owner_id(ctx.db_conn.clone(), auth.user.id.clone())
            .await
        {
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to find cart"})),
                )
            }
            Ok(Some(cart)) => cart,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Cart not found"})),
                )
            }
        };

    match cart.status {
        repository::cart::CartStatus::CheckedOut => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Cart is already checked out" })),
            )
        }
        repository::cart::CartStatus::NotCheckedOut => (),
    };

    if cart.items.len() == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Cart is empty!" })),
        );
    }

    let order = match repository::order::create(
        ctx.db_conn.clone(),
        repository::order::CreateOrderPayload {
            cart: cart.clone(),
            payment_method: payload.payment_method.clone(),
            delivery_address: payload.delivery_address.clone(),
            dispatch_rider_note: payload.dispatch_rider_note.clone(),
        },
    )
    .await
    {
        Ok(order) => order,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to checkout cart" })),
            );
        }
    };

    if let Err(_) = repository::cart::update_by_id(
        ctx.db_conn.clone(),
        cart.id.clone(),
        repository::cart::UpdateCartPayload {
            items: None,
            status: Some(repository::cart::CartStatus::CheckedOut),
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to checkout cart" })),
        );
    };

    (
        StatusCode::OK,
        Json(json!({ "message": "Cart checkedout successfully", "id": order.id })),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(get_carts))
        .route("/active", get(get_active_cart))
        .route("/:id", get(get_cart_by_id))
        .route("/active/checkout", post(checkout_active_cart))
        .route(
            "/active/items/:meal_id",
            put(set_meal_in_active_cart).delete(remove_meal_from_active_cart),
        )
}
