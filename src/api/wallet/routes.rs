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
    api::auth::middleware::Auth,
    repository,
    types::Context,
    utils::{self},
};

#[derive(Deserialize)]
struct CreateWalletAccountPayload {
    pub bvn: String,
    pub bank_code: String,
    pub account_number: String,
}

async fn create_bank_account(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateWalletAccountPayload>,
) -> impl IntoResponse {
    utils::wallet::create_bank_account(
        ctx.clone(),
        utils::wallet::CreateBankAccountPayload {
            bvn: payload.bvn,
            bank_code: payload.bank_code,
            account_number: payload.account_number,
            user: auth.user,
        },
    )
    .await;
}

async fn get_wallet_by_profile(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::wallet::find_by_owner_id(ctx.db_conn.clone(), auth.user.id).await {
        Ok(Some(wallet)) => (StatusCode::OK, Json(json!(wallet))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Wallet not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch wallet"})),
        ),
    }
}

async fn get_wallet_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::wallet::find_by_id(ctx.db_conn.clone(), id).await {
        Ok(Some(wallet)) => (StatusCode::OK, Json(json!(wallet))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Wallet not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch wallets"})),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/bank-account", post(create_bank_account))
        .route("/profile", get(get_wallet_by_profile))
        .route("/:id", get(get_wallet_by_id))
}
