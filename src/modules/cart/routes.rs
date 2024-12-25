use std::{collections::HashMap, sync::Arc};

use super::repository::{self, CartItem, CartItems, UpdateCartPayload};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use validator::Validate;

use crate::{
    modules::{
        auth::middleware::Auth,
        kitchen::repository::Kitchen,
        meal::{
            self,
            repository::{Meal, MealWithCartStatus},
        },
        order,
    },
    types::Context,
};

async fn get_active_cart(State(ctx): State<Arc<Context>>, auth: Auth) -> impl IntoResponse {
    #[derive(Debug, Serialize)]
    struct MealWithQuantity {
        meal: MealWithCartStatus,
        quantity: i32,
    }

    #[derive(Debug, Serialize)]
    struct KitchenWithMeals {
        kitchen: Kitchen,
        meals: Vec<MealWithQuantity>,
    }

    let cart =
        repository::find_active_full_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
            .await;

    tracing::info!("Cart: {:?}", cart);

    match cart {
        Ok(Some(cart)) => {
            tracing::info!("Cart items length: {:?}", cart.items.0.len());
            let kitchen_id_to_kitchen_map = cart
                .items
                .0
                .clone()
                .into_iter()
                .map(|item| item.kitchen.clone())
                .unique_by(|kitchen| kitchen.id.clone())
                .map(|kitchen| (kitchen.id.clone(), kitchen))
                .collect::<HashMap<String, Kitchen>>();

            let res = kitchen_id_to_kitchen_map
                .into_iter()
                .map(|(id, kitchen)| {
                    let meals = cart
                        .items
                        .0
                        .clone()
                        .into_iter()
                        .filter(|item| item.kitchen.id == id)
                        .map(|item| MealWithQuantity {
                            quantity: item.quantity,
                            meal: MealWithCartStatus {
                                id: item.meal.id,
                                name: item.meal.name,
                                description: item.meal.description,
                                rating: item.meal.rating,
                                original_price: item.meal.original_price,
                                price: item.meal.price,
                                likes: item.meal.likes,
                                cover_image: item.meal.cover_image,
                                is_available: item.meal.is_available,
                                kitchen_id: item.meal.kitchen_id,
                                created_at: item.meal.created_at,
                                updated_at: item.meal.updated_at,
                                in_cart: true,
                            },
                        })
                        .collect::<Vec<_>>();

                    KitchenWithMeals { kitchen, meals }
                })
                .collect::<Vec<_>>();

            (StatusCode::OK, Json(json!(res)))
        }
        Ok(None) => (StatusCode::OK, Json(json!([]))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch active cart" })),
        ),
    }
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
        match repository::find_active_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
            .await
        {
            Ok(None) => match repository::create(
                &ctx.db_conn.pool,
                repository::CreateCartPayload {
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

    let meal = match meal::repository::find_by_id(&ctx.db_conn.pool, meal_id).await {
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

    match repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id.clone(),
        repository::UpdateCartPayload {
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
        match repository::find_active_cart_by_owner_id(&ctx.db_conn.pool, auth.user.id.clone())
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

    let meal = match meal::repository::find_by_id(&ctx.db_conn.pool, meal_id).await {
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

    match repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id.clone(),
        repository::UpdateCartPayload {
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
pub struct CheckoutCartItemsByKitchenIdPayload {
    payment_method: order::repository::PaymentMethod,
    delivery_address: String,
    delivery_date: Option<u64>,
    dispatch_rider_note: String,
}

pub async fn checkout_cart_items_by_kitchen_id(
    State(ctx): State<Arc<Context>>,
    Path(kitchen_id): Path<String>,
    auth: Auth,
    Json(payload): Json<CheckoutCartItemsByKitchenIdPayload>,
) -> impl IntoResponse {
    let parsed_delivery_date = match payload
        .delivery_date
        .map(|d| chrono::NaiveDateTime::parse_from_str(&d.to_string(), "%s"))
        .transpose()
    {
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": err.to_string() })),
            )
        }
        Ok(x) => x,
    };

    match parsed_delivery_date.clone() {
        Some(delivery_date) => {
            if delivery_date < chrono::Utc::now().naive_utc() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Delivery date cannot be in the past" })),
                );
            }
        }
        None => (),
    }

    let cart = match repository::find_active_full_cart_by_owner_id(
        &ctx.db_conn.pool,
        auth.user.id.clone(),
    )
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

    let items_to_checkout = cart
        .items
        .0
        .into_iter()
        .filter(|item| item.kitchen.id == kitchen_id)
        .collect::<Vec<_>>();

    if items_to_checkout.len() == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No items to checkout!" })),
        );
    }

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

    let order = match order::repository::create(
        &mut *tx,
        order::repository::CreateOrderPayload {
            items: items_to_checkout,
            payment_method: payload.payment_method.clone(),
            delivery_address: payload.delivery_address.clone(),
            delivery_date: parsed_delivery_date.clone(),
            dispatch_rider_note: payload.dispatch_rider_note.clone(),
            kitchen_id,
            owner_id: auth.user.id.clone(),
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

    if let Err(_) = repository::update_by_id(
        &mut *tx,
        cart.id.clone(),
        repository::UpdateCartPayload {
            items: None,
            status: Some(repository::CartStatus::CheckedOut),
        },
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to checkout cart" })),
        );
    };

    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to checkout cart" })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({ "message": "Cart checkedout successfully", "id": order.id })),
    )
}

async fn remove_all_meals_from_active_cart_by_kitchen_id(
    State(ctx): State<Arc<Context>>,
    Path(kitchen_id): Path<String>,
    auth: Auth,
) -> impl IntoResponse {
    let cart = match repository::find_active_full_cart_by_owner_id(
        &ctx.db_conn.pool,
        auth.user.id.clone(),
    )
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

    let filtered_cart_items = cart
        .items
        .0
        .into_iter()
        .filter(|item| item.kitchen.id != kitchen_id)
        .map(|item| CartItem {
            meal_id: item.meal.id,
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();

    match repository::update_by_id(
        &ctx.db_conn.pool,
        cart.id,
        UpdateCartPayload {
            items: Some(CartItems(filtered_cart_items)),
            status: None,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Items removed from cart" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to remove items from cart"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(get_active_cart))
        .route(
            "/kitchens/:kitchen_id/checkout",
            post(checkout_cart_items_by_kitchen_id),
        )
        .route(
            "/items/:meal_id",
            put(set_meal_in_active_cart).delete(remove_meal_from_active_cart),
        )
        .route(
            "/kitchens/:kitchen_id",
            delete(remove_all_meals_from_active_cart_by_kitchen_id),
        )
}
