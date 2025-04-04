pub mod response {
    use axum::{
        extract::Json,
        http::StatusCode,
        response::{IntoResponse, Redirect},
    };
    use serde_json::json;

    pub enum Success {
        TokenGenerated(String),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::TokenGenerated(redirect_url) => Redirect::to(&redirect_url).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToGenerateToken,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToGenerateToken => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to generate token" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
