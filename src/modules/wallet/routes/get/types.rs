pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use super::super::super::super::repository::Wallet;

    pub enum Success {
        Wallet(Wallet),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Wallet(wallet) => (StatusCode::OK, Json(json!(wallet))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchWallet,
        WalletNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchWallet => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch wallet" })),
                )
                    .into_response(),
                Self::WalletNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Kitchen not wallet" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
