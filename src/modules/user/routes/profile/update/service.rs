use super::super::super::super::repository;
use super::types::{request, response};
use crate::types::Context;
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let update_payload = repository::UpdateUserPayload {
        email: payload.body.email,
        phone_number: payload.body.phone_number,
        first_name: payload.body.first_name,
        last_name: payload.body.last_name,
        has_kitchen: None,
        profile_picture: None,
    };

    repository::update_by_id(&ctx.db_conn.pool, payload.auth.user.id, update_payload)
        .await
        .map_err(|_| response::Error::FailedToUpdateUser)
        .map(|_| response::Success::UserUpdated)
}
