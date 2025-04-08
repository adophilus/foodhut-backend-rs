use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handler(
    auth: Auth,
    State(ctx): State<Arc<Context>>,
    Path(id): Path<String>,
    Json(body): Json<request::Body>,
) -> impl IntoResponse {
    service(ctx, request::Payload { id, body }, auth).await
}
