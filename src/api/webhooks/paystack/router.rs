use crate::types::Context;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha512;
use std::sync::Arc;

fn verify_header(ctx: Arc<Context>, header: String, body: String) -> bool {
    let mut mac = Hmac::<Sha512>::new_from_slice(ctx.payment.secret_key.as_str().as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_str().as_bytes());

    let calculated_signature = hex::encode(mac.clone().finalize().into_bytes());
    tracing::debug!("Calculated HMAC signature: '{}'", calculated_signature);

    match mac.verify_slice(header.as_str().as_bytes()) {
        Ok(_) => true,
        Err(err) => {
            tracing::warn!("Failed to verify HMAC header {}: {}", header, err);
            false
        }
    }
}

async fn handle_webhook(State(ctx): State<Arc<Context>>, req: Request) -> impl IntoResponse {
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
        }
    };
    // FIX: this doesn't work
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => {
            String::from_utf8(bytes.to_vec()).expect("Body couldn't be converted to string")
        }
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid body" })),
            )
        }
    };

    if !verify_header(ctx.clone(), x_paystack_signature_header, body) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unimplemented" })),
        );
    }

    (StatusCode::OK, Json(json!({ "error": "unimplemented" })))
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(handle_webhook))
}
