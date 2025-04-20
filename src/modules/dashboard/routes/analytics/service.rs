use super::types::{
    request::{self, GetAnalyticsFiltersType},
    response,
};
use crate::{
    modules::transaction::{self, repository::OrderFilter},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let filter_type = match payload.filters.r#type {
        request::GetAnalyticsFiltersType::Total => OrderFilter::Total,
        request::GetAnalyticsFiltersType::Vendor => OrderFilter::Vendor,
        request::GetAnalyticsFiltersType::Profit => OrderFilter::Profit,
    };

    let paginated_transactions = transaction::repository::find_many_for_orders(
        &ctx.db_conn.pool,
        payload.pagination,
        transaction::repository::FindManyForOrdersFilters {
            before: payload.filters.before,
            after: payload.filters.after,
            r#type: filter_type.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToFetchAnalytics)?;

    transaction::repository::get_total_transaction_volume_for_order(
        &ctx.db_conn.pool,
        transaction::repository::GetTotalTransactionVolumeForOrder {
            r#type: filter_type,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToFetchAnalytics)
    .map(|tv| response::Success::Analytics(tv, paginated_transactions))
}
