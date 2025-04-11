use super::{
    super::super::repository,
    types::{request, response},
};
use crate::{
    modules::{kitchen, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    match user::repository::is_admin(&payload.auth.user) {
        true => {
            repository::find_many(
                &ctx.db_conn.pool,
                payload.pagination,
                repository::FindManyFilters {
                    user_id: payload.filters.user_id,
                    before: payload.filters.before,
                    after: payload.filters.after,
                    kitchen_id: None,
                },
            )
            .await
        }
        false => match payload.filters.as_kitchen.is_some() {
            true => {
                kitchen::repository::find_by_owner_id(
                    &ctx.db_conn.pool,
                    payload.auth.user.id.clone(),
                )
                .await
                .map_err(|_| response::Error::FailedToFetchTransactions)?
                .ok_or(response::Error::KitchenNotFound)
                .map(|kitchen| {
                    repository::find_many(
                        &ctx.db_conn.pool,
                        payload.pagination,
                        repository::FindManyFilters {
                            user_id: Some(payload.auth.user.id.clone()),
                            before: payload.filters.before,
                            after: payload.filters.after,
                            kitchen_id: Some(kitchen.id),
                        },
                    )
                })?
                .await
            }
            false => {
                repository::find_many(
                    &ctx.db_conn.pool,
                    payload.pagination,
                    repository::FindManyFilters {
                        user_id: Some(payload.auth.user.id.clone()),
                        before: payload.filters.before,
                        after: payload.filters.after,
                        kitchen_id: None,
                    },
                )
                .await
            }
        },
    }
    .map(response::Success::Transactions)
    .map_err(|_| response::Error::FailedToFetchTransactions)
}
