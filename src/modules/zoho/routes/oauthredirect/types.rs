pub mod request {
    pub use crate::modules::auth::middleware::AdminAuth as Auth;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Params {
        #[serde(rename = "accounts-server")]
        pub account_server_url: String,
        pub code: String,
    }

    pub struct Payload {
        pub params: Params,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Deserialize, Serialize)]
    pub struct Tokens {
        pub access_token: String,
        pub refresh_token: String,
    }

    pub enum Success {
        Tokens(Tokens),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Tokens(tokens) => (StatusCode::OK, Json(json!(tokens))).into_response(),
            }
        }
    }

    pub enum Error {
        ServerError,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::ServerError => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Sorry, an error occurred" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
