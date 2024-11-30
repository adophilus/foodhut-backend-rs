use super::repository::{self, Filters};
use crate::{modules::auth::middleware::AdminAuth, types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde_json::json;
use std::sync::Arc;

async fn get_transaction_by_id(
    State(ctx): State<Arc<Context>>,
    _: AdminAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(tx)) => (StatusCode::OK, Json(json!(tx))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Transaction not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch transaction"})),
        ),
    }
}

async fn get_transactions(
    State(ctx): State<Arc<Context>>,
    _: AdminAuth,
    Query(filters): Query<Filters>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::find_many(&ctx.db_conn.pool, pagination, filters).await {
        Ok(transactions) => (StatusCode::OK, Json(json!(transactions))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch transactions"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/", get(get_transactions))
        .route("/:id", get(get_transaction_by_id))
}
