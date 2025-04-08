use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn handler(State(ctx): State<Arc<Context>>, auth: Auth) -> impl IntoResponse {
    service(ctx, request::Payload { auth }).await
}
