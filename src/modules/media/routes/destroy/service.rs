use super::types::{request, response};
use crate::types::Context;
use std::sync::Arc;

pub async fn service(_: Arc<Context>, payload: request::Payload) -> response::Response {
    std::fs::remove_file(format!("public/uploads/{}", payload.id))
        .map_err(|_| response::Error::FailedToDeleteMedia)
        .map(|_| response::Success::MediaDeleted)
}
