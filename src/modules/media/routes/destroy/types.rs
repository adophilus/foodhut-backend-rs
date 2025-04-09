pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        MediaDeleted,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MediaDeleted => (
                    StatusCode::OK,
                    Json(json!({ "message": "Media deleted successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToDeleteMedia,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToDeleteMedia => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Failed to delete media"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
