pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub account_number: String,
        pub bank_code: String,
    }

    pub struct Payload {
        pub body: Body,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BankAccountDetails {
        pub account_name: String,
    }

    pub enum Success {
        AccountDetails(BankAccountDetails),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AccountDetails(details) => {
                    (StatusCode::OK, Json(json!(details))).into_response()
                }
            }
        }
    }

    pub enum Error {
        FailedToFetchBankAccountDetails,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchBankAccountDetails => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch bank account details" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
