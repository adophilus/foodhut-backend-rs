use super::lib::handler;
use super::model;
use crate::types::Context;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::sync::Arc;

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

    let payload = match serde_json::de::from_str::<model::Event>(body.as_str()) {
        Ok(payload) => payload,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match payload {
        model::Event::TransactionSuccessful { amount, metadata } => {
            handler::successful_transaction(ctx.clone(), amount, metadata)
                .await
                .into_response()
        }
        // model::Event::CustomerIdentificationFailed(payload) => {
        //     handler::customer_identification_failed(ctx.clone(), payload)
        //         .await
        //         .into_response()
        // }
        // model::Event::CustomerIdentificationSuccessful(payload) => {
        //     handler::customer_identification_successful(ctx.clone(), payload)
        //         .await
        //         .into_response()
        // }
        model::Event::DedicatedAccountAssignmentSuccessful(payload) => {
            handler::dedicated_account_assignment_successful(ctx.clone(), payload)
                .await
                .into_response()
        }
        model::Event::DedicatedAccountAssignmentFailed(payload) => {
            handler::dedicated_account_assignment_failed(ctx.clone(), payload)
                .await
                .into_response()
        }
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(handle_webhook))
}
