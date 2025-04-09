use super::types::{request, response};
use crate::{modules::kitchen::repository, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    repository::unlike_by_id(&ctx.db_conn.pool, payload.id, payload.auth.user.id)
        .await
        .map_err(|_| response::Error::FailedToUnlikeMeal)
        .map(|_| response::Success::MealUnliked)
}
