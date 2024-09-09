use std::sync::Arc;

use crate::{api::auth::middleware::AdminAuth, repository};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::types::Context;

async fn get_info(State(ctx): State<Arc<Context>>, _: AdminAuth) -> impl IntoResponse {
    match repository::dashboard::get_total_resources(ctx.db_conn.clone()).await {
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
