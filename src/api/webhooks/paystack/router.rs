use crate::{repository, types::Context};
use crate::{types, utils};
use axum::Json;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha512;
use std::sync::Arc;

#[derive(Deserialize)]
struct CustomerIdentificationFailed {
    email: String,
    reason: String,
}

#[derive(Deserialize)]
struct CustomerIdentificationSuccessful {
    email: String,
}

#[derive(Deserialize)]
#[serde(tag = "event", content = "data")]
enum PaystackEvent {
    #[serde(rename = "charge.success")]
    TransactionSuccessful {
        amount: BigDecimal,
        metadata: utils::online::Metadata,
    },
    #[serde(rename = "customeridentification.success")]
    CustomerIdentificationSuccessful(CustomerIdentificationSuccessful),
    #[serde(rename = "customeridentification.failed")]
    CustomerIdentificationFailed(CustomerIdentificationFailed),
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

async fn handle_successful_transaction(
    ctx: Arc<types::Context>,
    amount: BigDecimal,
    metadata: utils::online::Metadata,
) -> impl IntoResponse {
    let order = match repository::order::find_by_id(ctx.db_conn.clone(), metadata.order_id).await {
        Ok(Some(order)) => order,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if amount / BigDecimal::from_u8(100).expect("Invalid primitive value to convert from")
        < order.total
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let cart = match repository::cart::find_by_id(ctx.db_conn.clone(), order.cart_id.clone()).await
    {
        Ok(Some(cart)) => cart,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Err(_) = repository::transaction::create(
        ctx.db_conn.clone(),
        repository::transaction::CreatePayload::Online(
            repository::transaction::CreateOnlineTransactionPayload {
                amount: order.total.clone(),
                r#type: repository::transaction::TransactionType::Debit,
                note: Some(format!("Paid for order {}", order.id.clone())),
                user_id: cart.owner_id.clone(),
            },
        ),
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Err(_) = utils::payment::confirm_payment_for_order(ctx.clone(), order).await {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}

async fn handle_customer_identification_failed(
    ctx: Arc<types::Context>,
    payload: CustomerIdentificationFailed,
) -> impl IntoResponse {
    let user =
        match repository::user::find_by_email(ctx.db_conn.clone(), payload.email.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return StatusCode::NOT_FOUND;
            }
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

    utils::notification::send(
        ctx.clone(),
        utils::notification::Notification::customer_identification_failed(user, payload.reason),
        utils::notification::Backend::Email,
    )
    .await;

    StatusCode::OK
}

async fn handle_customer_identification_successful(
    ctx: Arc<types::Context>,
    payload: CustomerIdentificationSuccessful
) -> impl IntoResponse {
    unimplemented!()
}

async fn handle_webhook(State(ctx): State<Arc<Context>>, req: Request) -> Response {
    let x_paystack_signature_header = match req.headers().get("X-PAYSTACK-SIGNATURE") {
        Some(header) => String::from(
            header
                .to_str()
                .expect("Header couldn't be converted to string"),
        ),
        None => {
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => {
            String::from_utf8(bytes.to_vec()).expect("Body couldn't be converted to string")
        }
        Err(_) => {
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if !verify_header(ctx.clone(), x_paystack_signature_header, body.clone()) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    tracing::debug!("Trying to parse body: {}", body.as_str());

    let payload = match serde_json::de::from_str::<PaystackEvent>(body.as_str()) {
        Ok(payload) => payload,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match payload {
        PaystackEvent::TransactionSuccessful { amount, metadata } => {
            handle_successful_transaction(ctx.clone(), amount, metadata)
                .await
                .into_response()
        }
        PaystackEvent::CustomerIdentificationFailed(payload) => {
            handle_customer_identification_failed(ctx.clone(), payload)
                .await
                .into_response()
        }
        PaystackEvent::CustomerIdentificationSuccessful(payload) => {
            handle_customer_identification_successful(ctx.clone(), payload)
                .await
                .into_response()
        }
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(handle_webhook))
}
