use super::types::{request, response};
use crate::{
    modules::{transaction::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    match user::repository::is_admin(&payload.auth.user) {
        true => repository::find_by_id(&ctx.db_conn.pool, payload.id).await,
        false => {
            repository::find_by_id_and_user_id(
                &ctx.db_conn.pool,
                payload.id,
                payload.auth.user.id.clone(),
            )
            .await
        }
    }
    .map_err(|_| response::Error::FailedToFetchTransaction)?
    .ok_or(response::Error::TransactionNotFound)
    .map(response::Success::Transaction)
}
