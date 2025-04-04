pub mod request {
    use crate::utils::pagination::Pagination;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub enum GetAnalyticsFiltersType {
        #[serde(rename = "total")]
        Total,
        #[serde(rename = "vendor")]
        Vendor,
        #[serde(rename = "profit")]
        Profit,
    }

    #[derive(Deserialize)]
    pub struct Filters {
        pub r#type: GetAnalyticsFiltersType,
        pub before: Option<u64>,
        pub after: Option<u64>,
    }

    pub struct Payload {
        pub filters: Filters,
        pub pagination: Pagination,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::{
        modules::transaction::repository::{TotalTransactionVolume, Transaction},
        utils::pagination::Paginated,
    };

    pub enum Success {
        Analytics(TotalTransactionVolume, Paginated<Transaction>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Analytics(tv, tx) => (
                    StatusCode::OK,
                    Json(json!({"total": tv.total_transaction_volume, "data": tx})),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchInfo,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchInfo => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch info"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
