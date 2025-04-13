use super::types::{request, response};
use crate::{
    modules::{user, zoho},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let user = user::repository::find_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToRegisterUser)?
        .ok_or(response::Error::UserNotFound)?;

    zoho::service::register_user(ctx.clone(), user)
        .await
        .map_err(|_| response::Error::FailedToRegisterUser)
        .map(|_| response::Success::UserRegistered)
}
