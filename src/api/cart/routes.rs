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
    repository::{self, cart::CartItems},
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
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Cart created!"})),
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

    if repository::cart::is_owner(auth.user.clone(), cart.clone()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this cart"})),
        );
    }

    (StatusCode::OK, Json(json!(cart)))
}

#[derive(Deserialize, Validate)]
pub struct UpdateCartPayload {
    items: CartItems,
}

async fn update_cart_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<UpdateCartPayload>,
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

    if repository::cart::is_owner(auth.user.clone(), cart.clone()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not the owner of this cart"})),
        );
    }

    match repository::cart::update_by_id(
        ctx.db_conn.clone(),
        id,
        repository::cart::UpdateCartPayload {
            items: Some(payload.items),
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

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_cart).get(get_carts))
        .route("/:id", get(get_cart_by_id).patch(update_cart_by_id))
}
