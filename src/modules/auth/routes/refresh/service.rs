use super::types::{request, response};
use crate::{modules::auth::service, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    service::auth::regenerate_tokens_for_session(ctx.clone(), payload.token)
        .await
        .map_err(|_| response::Error::FailedToRefreshTokens)
        .map(|session| response::Success::Tokens((session.access_token, session.refresh_token)))
}
