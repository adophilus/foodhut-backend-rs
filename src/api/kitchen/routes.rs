use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api::auth::middleware::Auth, repository, types::Context, utils::pagination::Pagination,
};

#[derive(Deserialize)]
struct CreateKitchenPayload {
    pub name: String,
    pub address: String,
    pub phone_number: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub opening_time: String,
    pub closing_time: String,
    pub preparation_time: String,
    pub delivery_time: String,
}

async fn create_kitchen(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateKitchenPayload>,
) -> impl IntoResponse {
    match repository::kitchen::create(
        ctx.db_conn.clone(),
        repository::kitchen::CreateKitchenPayload {
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
            Json(json!({ "message": "Kitchen created!"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Kitchen creation failed"})),
        ),
    }
}

async fn get_kitchens(
    State(ctx): State<Arc<Context>>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::kitchen::find_many(ctx.db_conn.clone(), pagination.clone()).await {
        Ok(paginated_kitchens) => (StatusCode::OK, Json(json!(paginated_kitchens))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchens"})),
        ),
    }
}

async fn get_kitchen_by_profile(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::kitchen::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
        Ok(Some(kitchen)) => (StatusCode::OK, Json(json!(kitchen))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Kitchen not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchen"})),
        ),
    }
}

async fn get_kitchen_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::kitchen::find_by_id(ctx.db_conn.clone(), id).await {
        Ok(Some(kitchen)) => (StatusCode::OK, Json(json!(kitchen))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Kitchen not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch kitchens"})),
        ),
    }
}

async fn fetch_kitchen_types() -> impl IntoResponse {
    Json(json!(["Chinese", "Cuisine", "Fast Food", "Local"]))
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", post(create_kitchen).get(get_kitchens))
        .route("/profile", get(get_kitchen_by_profile))
        .route("/:id", get(get_kitchen_by_id))
        .route("/types", get(fetch_kitchen_types))
}
