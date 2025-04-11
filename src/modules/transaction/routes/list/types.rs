pub mod request {
    pub use crate::{modules::auth::middleware::Auth, utils::pagination::Pagination};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Filters {
        pub user_id: Option<String>,
        pub before: Option<u64>,
        pub after: Option<u64>,
        pub as_kitchen: Option<bool>,
    }

    pub struct Payload {
        pub pagination: Pagination,
        pub filters: Filters,
        pub auth: Auth,
    }
}

pub mod response {
    use super::super::super::super::repository::Transaction;
    use crate::utils::pagination::Paginated;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Transactions(Paginated<Transaction>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Transactions(ads) => (StatusCode::OK, Json(json!(ads))).into_response(),
            }
        }
    }

    pub enum Error {
        KitchenNotFound,
        FailedToFetchTransactions,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Kitchen not found" })),
                )
                    .into_response(),
                Self::FailedToFetchTransactions => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch transactions" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
