pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        AdDeleted,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AdDeleted => (
                    StatusCode::OK,
                    Json(json!({ "message": "Ad deleted successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        AdNotFound,
        FailedToDeleteAd,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToDeleteAd => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete ad" })),
                )
                    .into_response(),
                Self::AdNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Ad not found" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
