use super::service;
use crate::types::Context;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, Router},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
struct ExchangeZohoGrantCodeForAuthTokensQueryParams {
    #[serde(rename = "accounts-server")]
    account_server_url: String,
    code: String,
}

async fn exchange_zoho_grant_code_for_auth_tokens(
    State(ctx): State<Arc<Context>>,
    Query(params): Query<ExchangeZohoGrantCodeForAuthTokensQueryParams>,
) -> impl IntoResponse {
    tracing::info!("Grant code: {}", &params.code);

    match service::exchange_grant_code_for_tokens(
        ctx,
        service::ExchangePayload {
            grant_code: params.code,
            account_server_url: params.account_server_url,
        },
    )
    .await
    {
        Ok(tokens) => (StatusCode::OK, Json(json!(tokens))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Sorry an unexpected error occurred"
            })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route(
        "/oauthredirect",
        get(exchange_zoho_grant_code_for_auth_tokens),
    )
}
