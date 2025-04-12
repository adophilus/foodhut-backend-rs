use super::{
    super::super::repository,
    types::{request, response},
};
use crate::{modules::kitchen, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let wallet = match payload.filters.as_kitchen.is_some() {
        true => {
            let kitchen =
                kitchen::repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id)
                    .await
                    .map_err(|_| response::Error::FailedToFetchWallet)?
                    .ok_or(response::Error::KitchenNotCreated)?;

            repository::find_by_kitchen_id(&ctx.db_conn.pool, kitchen.id)
                .await
                .map_err(|_| response::Error::FailedToFetchWallet)?
                .ok_or(response::Error::FailedToFetchWallet)?
        }
        false => repository::find_by_owner_id(&ctx.db_conn.pool, payload.auth.user.id)
            .await
            .map_err(|_| response::Error::FailedToFetchWallet)?
            .ok_or(response::Error::FailedToFetchWallet)?,
    };

    Ok(response::Success::Wallet(wallet))
}
