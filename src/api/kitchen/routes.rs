use std::sync::Arc;

use axum::{routing::post, Json, Router};
use serde::Deserialize;

use crate::types::{ApiResponse, Context};

#[derive(Deserialize)]
struct CreateKitchenPayload {
    pub name: String,
    pub address: String,
    pub phone_number: String,
    pub type_: String,
    pub opening_time: String,
    pub closing_time: String,
    // pub cover_image: String,
}

async fn create_kitchen(
    Json(payload): Json<CreateKitchenPayload>,
) -> ApiResponse<&'static str, &'static str> {
    ApiResponse::ok("Kitchen created!")
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(create_kitchen))
}
