pub mod request {
    pub use crate::utils::pagination::Pagination;

    pub struct Payload {
        pub pagination: Pagination,
    }
}

pub mod response {
    use super::super::super::super::repository::DbPaystackBank;
    use crate::utils::pagination::Paginated;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Banks(Paginated<DbPaystackBank>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Banks(banks) => (StatusCode::OK, Json(json!(banks))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchBanks,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchBanks => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to fetch banks"
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
