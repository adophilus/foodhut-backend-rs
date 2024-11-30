use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use super::repository;
use crate::{modules::auth::middleware::AdminAuth, types::Context};

async fn get_info(State(ctx): State<Arc<Context>>, _: AdminAuth) -> impl IntoResponse {
    match repository::get_total_resources(&ctx.db_conn.pool).await {
        Ok(resources) => (StatusCode::OK, Json(json!(resources))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch info"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/info", get(get_info))
}
