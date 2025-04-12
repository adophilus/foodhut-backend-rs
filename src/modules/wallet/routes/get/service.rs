use super::{
    super::super::repository,
    types::{request, response},
};
use crate::types::Context;
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::find_by_id(&ctx.db_conn.pool, payload.id)
        .await
        .map_err(|_| response::Error::FailedToFetchWallet)?
        .ok_or(response::Error::WalletNotFound)
        .map(response::Success::Wallet)
}
