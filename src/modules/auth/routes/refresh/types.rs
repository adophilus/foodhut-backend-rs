pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub token: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Tokens((String, String)),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Tokens((access_token, refresh_token)) => (
                    StatusCode::OK,
                    Json(json!({
                        "access_token": access_token,
                        "refresh_token": refresh_token,
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToRefreshTokens,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToRefreshTokens => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to refresh tokens" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
