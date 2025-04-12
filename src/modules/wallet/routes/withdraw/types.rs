pub mod request {
    use crate::modules::auth::middleware::Auth;
    use bigdecimal::BigDecimal;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub account_number: String,
        pub bank_code: String,
        pub account_name: String,
        pub amount: BigDecimal,
        pub as_kitchen: bool,
    }

    pub struct Payload {
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        WithdrawalPlaced,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::WithdrawalPlaced => (
                    StatusCode::OK,
                    Json(json!({ "message": "Withdrawal request placed" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToWithdrawFunds,
        InsufficientFunds,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::InsufficientFunds => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Insufficient funds" })),
                )
                    .into_response(),
                Self::FailedToWithdrawFunds => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to place withdrawal request" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
