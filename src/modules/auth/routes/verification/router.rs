use super::send_otp;
use super::verify_otp;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/send-otp", send_otp::get_router())
        .nest("/verify-otp", verify_otp::get_router())
}
