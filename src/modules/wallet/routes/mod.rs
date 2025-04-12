mod bank_account;
mod banks;
mod get;
mod profile;
mod top_up;
mod withdraw;

use crate::types::Context;
use axum::routing::Router;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .nest("/", banks::get_router())
        .nest("/", bank_account::get_router())
        .nest("/", get::get_router())
        .nest("/", profile::get_router())
        .nest("/", top_up::get_router())
        .nest("/", withdraw::get_router())
}
