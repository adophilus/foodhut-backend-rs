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
    repository::like_by_id(&ctx.db_conn.pool, payload.id, auth.user.id)
        .await
        .map_err(|_| response::Error::FailedToLikeKitchen)
        .map(|_| response::Success::KitchenLiked)
}
