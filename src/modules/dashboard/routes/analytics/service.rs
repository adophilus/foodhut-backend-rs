use super::types::{
    request::{self, GetAnalyticsFiltersType},
    response,
};
use crate::{
    modules::transaction::{self, repository::FindManyForOrdersType},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let paginated_transactions = match payload.filters.r#type {
        GetAnalyticsFiltersType::Total => transaction::repository::find_many_for_orders(
            &ctx.db_conn.pool,
            payload.pagination,
            transaction::repository::FindManyForOrdersFilters {
                before: payload.filters.before,
                after: payload.filters.after,
                r#type: FindManyForOrdersType::Total,
            },
        )
        .await
        .map_err(|_| response::Error::FailedToFetchAnalytics)?,
        GetAnalyticsFiltersType::Vendor => transaction::repository::find_many_for_orders(
            &ctx.db_conn.pool,
            payload.pagination,
            transaction::repository::FindManyForOrdersFilters {
                before: payload.filters.before,
                after: payload.filters.after,
                r#type: FindManyForOrdersType::Vendor,
            },
        )
        .await
        .map_err(|_| response::Error::FailedToFetchAnalytics)?,
        _ => unimplemented!(),
    };

    transaction::repository::get_total_transaction_volume(&ctx.db_conn.pool)
        .await
        .map_err(|_| response::Error::FailedToFetchAnalytics)
        .map(|tv| response::Success::Analytics(tv, paginated_transactions))
}
