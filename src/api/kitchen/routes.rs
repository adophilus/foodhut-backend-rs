use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api::auth::middleware::Auth,
    repository,
    types::{ApiResponse, Context, Pagination},
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

async fn get_kitchens(pagination: Pagination) -> impl IntoResponse {
    // let kitchens = repository::kitchen::find_many(pagination);

    (
        StatusCode::OK,
        Json(json!({
            "items": [],
            "meta": {
                "page": pagination.page,
                "per_page": pagination.per_page,
            }
        })),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(create_kitchen).get(get_kitchens))
}
