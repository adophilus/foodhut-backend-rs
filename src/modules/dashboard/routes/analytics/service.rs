use super::types::{request, response};
use crate::{modules::transaction, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let paginated_transactions = transaction::repository::find_many(
        &ctx.db_conn.pool,
        payload.pagination,
        transaction::repository::FindManyFilters {
            user_id: None,
            before: payload.filters.before,
            after: payload.filters.after,
            kitchen_id: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToFetchInfo)?;

    transaction::repository::get_total_transaction_volume(&ctx.db_conn.pool)
        .await
        .map_err(|_| response::Error::FailedToFetchInfo)
        .map(|tv| response::Success::Analytics(tv, paginated_transactions))
}
