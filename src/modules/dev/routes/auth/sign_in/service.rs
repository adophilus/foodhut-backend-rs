use super::types::{request, response};
use crate::{
    modules::{auth, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let user =
        user::repository::find_by_phone_number(&ctx.db_conn.pool, payload.phone_number.clone())
            .await
            .map_err(|_| response::Error::FailedToFetchUser)?
            .ok_or(response::Error::UserNotFound)?;

    auth::service::auth::create_session(&ctx.db_conn.pool, user.id)
        .await
        .map_err(|_| response::Error::FailedToCreateSession)
        .map(response::Success::Tokens)
}
