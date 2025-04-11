use super::{service::service, types::request};
use axum::response::IntoResponse;

pub async fn handler(auth: request::Auth) -> impl IntoResponse {
    service(request::Payload { auth }).await
}
