pub mod response {
    use crate::modules::dashboard::repository::DatabaseInfo;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Info(DatabaseInfo),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Info(info) => (StatusCode::OK, Json(json!(info))).into_response(),
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
