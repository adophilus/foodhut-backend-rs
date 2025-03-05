use std::sync::Arc;

use super::repository;
use crate::{
    modules::{
        auth::middleware::{AdminAuth, Auth},
        notification, transaction,
    },
    types::Context,
    utils::pagination::Pagination,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

async fn get_info(State(ctx): State<Arc<Context>>, _: AdminAuth) -> impl IntoResponse {
    match repository::get_total_resources(&ctx.db_conn.pool).await {
        Ok(resources) => (StatusCode::OK, Json(json!(resources))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch info"})),
        ),
    }
}

#[derive(Deserialize)]
pub enum GetAnalyticsFiltersType {
    #[serde(rename = "total")]
    Total,
    #[serde(rename = "vendor")]
    Vendor,
    #[serde(rename = "profit")]
    Profit,
}

#[derive(Deserialize)]
struct GetAnalyticsFilters {
    r#type: GetAnalyticsFiltersType,
    before: Option<u64>,
    after: Option<u64>,
}

async fn get_analytics(
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<GetAnalyticsFilters>,
    _: AdminAuth,
    pagination: Pagination,
) -> impl IntoResponse {
    let paginated_transactions = match transaction::repository::find_many(
        &ctx.db_conn.pool,
        pagination,
        transaction::repository::FindManyFilters {
            user_id: None,
            before: filters.before,
            after: filters.after,
            kitchen_id: None,
        },
    )
    .await
    {
        Ok(paginated_transactions) => paginated_transactions,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch info"})),
            )
        }
    };

    let tv = match transaction::repository::get_total_transaction_volume(&ctx.db_conn.pool).await {
        Ok(tv) => tv,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch info"})),
            )
        }
    };

    (
        StatusCode::OK,
        Json(json!({"total": tv.total_transaction_volume, "data": paginated_transactions})),
    )
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/info", get(get_info))
        .route("/analytics", get(get_analytics))
}
