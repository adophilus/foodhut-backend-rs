pub mod request {
    pub use crate::modules::auth::middleware::Auth;

    pub struct Payload {
        pub id: String,
        pub auth: Auth,
    }
}

pub mod response {
    use super::super::super::super::repository::Transaction;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Transaction(Transaction),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Transaction(tx) => (StatusCode::OK, Json(json!(tx))).into_response(),
            }
        }
    }

    pub enum Error {
        TransactionNotFound,
        FailedToFetchTransaction,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::TransactionNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Transaction not found" })),
                )
                    .into_response(),
                Self::FailedToFetchTransaction => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch transaction"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
