use super::{service::service, types::request};
use crate::{modules::auth::middleware::Auth, types::Context};
use axum::{extract::State, response::IntoResponse};
use axum_typed_multipart::TypedMultipart;
use std::sync::Arc;

pub async fn handler(
    State(ctx): State<Arc<Context>>,
    auth: Auth,
    TypedMultipart(body): TypedMultipart<request::Body>,
) -> impl IntoResponse {
    service(ctx, request::Payload { auth, body }).await
}
