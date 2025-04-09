use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, kitchen::repository, user},
    types::Context,
};
use std::sync::Arc;

pub async fn service(
    ctx: Arc<Context>,
    auth: Option<Auth>,
    payload: request::Payload,
) -> response::Response {
    match auth {
        Some(auth) => {
            if user::repository::is_admin(&auth.user) {
                repository::find_many_as_admin(
                    &ctx.db_conn.pool,
                    payload.pagination.clone(),
                    payload.filters,
                )
                .await
            } else {
                repository::find_many_as_user(
                    &ctx.db_conn.pool,
                    payload.pagination.clone(),
                    payload.filters,
                )
                .await
            }
        }
        None => {
            repository::find_many_as_user(&ctx.db_conn.pool, payload.pagination.clone(), payload.filters)
                .await
        }
    }
    .map_err(|_|response::Error::FailedToFetchKitchens)
    .map(response::Success::Kitchens)
}
