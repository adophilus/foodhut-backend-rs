use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: Option<Auth>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id, auth }).await
}
