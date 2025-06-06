use super::types::{request, response};
use crate::{modules::meal::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::like_by_id(&ctx.db_conn.pool, payload.id, payload.auth.user.id)
        .await
        .map_err(|_| response::Error::FailedToLikeMeal)
        .map(|_| response::Success::MealLiked)
}
