pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        KitchenUnverified,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenUnverified => (
                    StatusCode::OK,
                    Json(json!({ "message": "Kitchen unverified successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUnverifyKitchen,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUnverifyKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to unverify kitchen" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
