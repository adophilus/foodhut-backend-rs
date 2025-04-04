use super::service::service;
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(auth: Auth, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    service(ctx, auth).await
}
