use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;

use crate::{
    api::auth::middleware::Auth,
    repository,
    types::{ApiResponse, Context},
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
) -> ApiResponse<&'static str, &'static str> {
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
        Ok(_) => ApiResponse::ok("Kitchen created!"),
        Err(_) => ApiResponse::err("Kitchen creation failed"),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(create_kitchen))
}
