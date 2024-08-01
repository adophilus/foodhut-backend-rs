use crate::utils;
use crate::{repository, types::Context};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use bigdecimal::BigDecimal;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha512;
use std::sync::Arc;

#[derive(Deserialize)]
enum PaystackEvent {
    TransactionSuccessful(TransactionSuccessful),
}

#[derive(Deserialize)]
struct TransactionSuccessfulData {
    amount: BigDecimal,
    metadata: utils::online::Metadata,
}

#[derive(Deserialize)]
struct TransactionSuccessful {
    event: String,
    data: TransactionSuccessfulData,
}

fn verify_header(ctx: Arc<Context>, header: String, body: String) -> bool {
    let mut mac = Hmac::<Sha512>::new_from_slice(ctx.payment.secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());

    match mac.verify_slice(
        hex::decode(header.clone())
            .expect("Invalid hex header")
            .as_ref(),
    ) {
        Ok(_) => true,
        Err(err) => {
            tracing::warn!("Failed to verify header {}: {}", header, err);
            false
        }
    }
}

async fn handle_webhook(State(ctx): State<Arc<Context>>, req: Request) -> Response {
    let x_paystack_signature_header = match req.headers().get("X-PAYSTACK-SIGNATURE") {
        Some(header) => String::from(
            header
                .to_str()
                .expect("Header couldn't be converted to string"),
        ),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid header" })),
            )
                .into_response();
        }
    };
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => {
            String::from_utf8(bytes.to_vec()).expect("Body couldn't be converted to string")
        }
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid body" })),
            )
                .into_response();
        }
    };

    if !verify_header(ctx.clone(), x_paystack_signature_header, body.clone()) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let payload = match serde_json::de::from_str::<PaystackEvent>(body.as_str()) {
        Ok(payload) => payload,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match payload {
        PaystackEvent::TransactionSuccessful(event) => {
            let order = match repository::order::find_by_id(
                ctx.db_conn.clone(),
                event.data.metadata.order_id,
            )
            .await
            {
                Ok(Some(order)) => order,
                Ok(None) => return StatusCode::NOT_FOUND.into_response(),
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            if let Err(_) = utils::payment::confirm_payment_for_order(ctx.clone(), order).await {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    StatusCode::OK.into_response()
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(handle_webhook))
}
