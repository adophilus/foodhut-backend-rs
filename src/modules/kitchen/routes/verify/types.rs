pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        KitchenVerified,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenVerified => (
                    StatusCode::OK,
                    Json(json!({ "message": "Kitchen verified successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToVerifyKitchen,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToVerifyKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to verify kitchen" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
