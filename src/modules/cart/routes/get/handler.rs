use super::service::service;
use crate::modules::auth::middleware::Auth;
use crate::types::Context;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(State(ctx): State<Arc<Context>>, auth: Auth) -> impl IntoResponse {
    service(ctx, auth).await
}
