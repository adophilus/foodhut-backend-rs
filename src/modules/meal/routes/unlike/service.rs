use super::types::{request, response};
use crate::{
    modules::{auth::middleware::Auth, kitchen::repository},
    types::Context,
};
use std::sync::Arc;

pub async fn service(
    ctx: Arc<Context>,
    payload: request::Payload,
    auth: Auth,
) -> response::Response {
    repository::unlike_by_id(&ctx.db_conn.pool, payload.id, auth.user.id)
        .await
        .map_err(|_| response::Error::FailedToUnlikeKitchen)
        .map(|_| response::Success::KitchenUnliked)
}
