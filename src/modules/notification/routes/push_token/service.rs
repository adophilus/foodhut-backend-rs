use super::types::{request, response};
use crate::{modules::notification::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::push_token::create(
        &ctx.db_conn.pool,
        repository::push_token::CreatePushTokenPayload {
            token: payload.body.token,
            user_id: payload.auth.user.id,
        },
    )
    .await
    .map(|_| response::Success::PushTokenCreated)
    .map_err(|_| response::Error::FailedToCreatePushToken)
}
