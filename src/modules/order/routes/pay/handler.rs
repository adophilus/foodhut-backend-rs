use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Json(payload): Json<request::Payload>,
) -> impl IntoResponse {
    service(ctx, auth, payload).await
}
