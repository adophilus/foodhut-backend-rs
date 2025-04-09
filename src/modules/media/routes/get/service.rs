use super::types::{request, response};
use crate::types::Context;
use std::sync::Arc;

pub async fn service(_: Arc<Context>, payload: request::Payload) -> response::Response {
    Ok(response::Success::Media(payload.id))
}
