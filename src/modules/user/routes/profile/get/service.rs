use super::types::{request, response};

pub async fn service(payload: request::Payload) -> response::Response {
    Ok(response::Success::User(payload.auth.user))
}
