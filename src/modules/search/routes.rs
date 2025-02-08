use crate::{types::Context, utils::pagination::Pagination};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, Router},
    Json,
};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use super::repository::{find_many_meals_and_kitchens, FindManyMealsAndKitchenFilters};

async fn search_meal_and_order(
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<FindManyMealsAndKitchenFilters>,
    pagination: Pagination,
) -> impl IntoResponse {
    match find_many_meals_and_kitchens(&ctx.db_conn.pool, filters, pagination).await {
        Ok(meals_or_orders) => (StatusCode::OK, Json(json!(meals_or_orders))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch meals and kitchens" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", get(search_meal_and_order))
}
