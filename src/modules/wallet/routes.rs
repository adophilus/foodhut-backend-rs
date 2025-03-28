use std::sync::Arc;

use super::{repository, service};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::json;

use crate::{
    modules::{auth::middleware::Auth, kitchen},
    types::Context,
    utils::pagination::Pagination,
};

#[derive(Deserialize)]
struct CreateVirtualAccountPayload {
    pub bvn: String,
    pub bank_code: String,
    pub account_number: String,
}

async fn create_bank_account(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateVirtualAccountPayload>,
) -> impl IntoResponse {
    match service::request_virtual_account(
        ctx.clone(),
        service::RequestVirtualAccountPayload {
            bvn: payload.bvn,
            bank_code: payload.bank_code,
            account_number: payload.account_number,
            user: auth.user,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Verification request sent" })),
        ),
        Err(service::CreationError::CreationFailed(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err })),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to request verification" })),
        ),
    }
}

async fn get_bank_account_details(
    _: Auth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<service::GetBankAccountDetailsPayload>,
) -> impl IntoResponse {
    match service::get_bank_account_details(ctx.clone(), payload).await {
        Ok(details) => (StatusCode::OK, Json(json!(details))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch bank account details" })),
        ),
    }
}

async fn get_banks(
    _: Auth,
    State(ctx): State<Arc<Context>>,
    pagination: Pagination,
) -> impl IntoResponse {
    match repository::find_many_banks(&ctx.db_conn.pool, pagination).await {
        Ok(banks) => (StatusCode::OK, Json(json!(banks))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch banks"
            })),
        ),
    }
}

#[derive(Deserialize)]
struct GetWalletByProfileFilters {
    as_kitchen: Option<bool>,
}

async fn get_wallet_by_profile(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Query(filters): Query<GetWalletByProfileFilters>,
) -> impl IntoResponse {
    let maybe_wallet = if filters.as_kitchen.unwrap_or(false) {
        let kitchen =
            match kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id).await {
                Ok(Some(kitchen)) => kitchen,
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({ "error": "Kitchen not found" })),
                    )
                }
            };
        repository::find_by_kitchen_id(&ctx.db_conn.pool, kitchen.id).await
    } else {
        repository::find_by_owner_id(&ctx.db_conn.pool, auth.user.id).await
    };

    match maybe_wallet {
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
    match repository::find_by_id(&ctx.db_conn.pool, id).await {
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

#[derive(Deserialize)]
struct CreateTopupInvoicePayload {
    amount: BigDecimal,
}

async fn create_topup_invoice(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    Json(payload): Json<CreateTopupInvoicePayload>,
) -> impl IntoResponse {
    match service::create_topup_invoice(
        ctx.clone(),
        service::CreateTopupInvoicePayload {
            amount: payload.amount,
            user: auth.user,
        },
    )
    .await
    {
        Ok(url) => (StatusCode::OK, Json(json!({ "url": url }))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create topup invoice" })),
        ),
    }
}

#[derive(Deserialize)]
struct WithdrawFundsPayload {
    account_number: String,
    bank_code: String,
    account_name: String,
    amount: BigDecimal,
    as_kitchen: bool,
}

async fn withdraw_funds(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<WithdrawFundsPayload>,
) -> impl IntoResponse {
    match service::withdraw_funds(
        ctx.clone(),
        service::WithdrawFundsPayload {
            account_number: payload.account_number,
            bank_code: payload.bank_code,
            account_name: payload.account_name,
            amount: payload.amount,
            user: auth.user,
            as_kitchen: payload.as_kitchen,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Withdrawal request placed" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to request withdrawal" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/bank-account", post(create_bank_account))
        .route("/bank-account/details", post(get_bank_account_details))
        .route("/banks", get(get_banks))
        .route("/profile", get(get_wallet_by_profile))
        .route("/:id", get(get_wallet_by_id))
        .route("/top-up", post(create_topup_invoice))
        .route("/withdraw", post(withdraw_funds))
}
