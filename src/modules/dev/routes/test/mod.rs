mod email;
mod push_notification;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/email", email::get_router())
        .nest("/push-notification", push_notification::get_router())
}
