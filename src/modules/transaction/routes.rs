use super::repository;
use crate::{
    modules::{
        auth::middleware::{AdminAuth, Auth},
        kitchen, user,
    },
    types::Context,
    utils::pagination::Pagination,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

async fn get_transaction_by_id(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let transaction_result = if user::repository::is_admin(&auth.user) {
        repository::find_by_id(&ctx.db_conn.pool, id).await
    } else {
        repository::find_by_id_and_user_id(&ctx.db_conn.pool, id, auth.user.id.clone()).await
    };

    match transaction_result {
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

#[derive(Deserialize)]
struct GetTransactionFilters {
    user_id: Option<String>,
    before: Option<u64>,
    after: Option<u64>,
    as_kitchen: Option<bool>,
}

async fn get_transactions(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Query(filters): Query<GetTransactionFilters>,
    pagination: Pagination,
) -> impl IntoResponse {
    let transactions_result = if user::repository::is_admin(&auth.user) {
        repository::find_many(
            &ctx.db_conn.pool,
            pagination,
            repository::FindManyFilters {
                user_id: filters.user_id,
                before: filters.before,
                after: filters.after,
                kitchen_id: None,
            },
        )
        .await
    } else {
        if filters.as_kitchen.unwrap_or(false) {
            let kitchen = match kitchen::repository::find_by_owner_id(
                &ctx.db_conn.pool,
                auth.user.id.clone(),
            )
            .await
            {
                Ok(Some(kitchen)) => kitchen,
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({ "error": "Kitchen not found"})),
                    )
                }
            };

            repository::find_many(
                &ctx.db_conn.pool,
                pagination,
                repository::FindManyFilters {
                    user_id: Some(auth.user.id.clone()),
                    before: filters.before,
                    after: filters.after,
                    kitchen_id: Some(kitchen.id),
                },
            )
            .await
        } else {
            repository::find_many(
                &ctx.db_conn.pool,
                pagination,
                repository::FindManyFilters {
                    user_id: Some(auth.user.id.clone()),
                    before: filters.before,
                    after: filters.after,
                    kitchen_id: None,
                },
            )
            .await
        }
    };

    match transactions_result {
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
