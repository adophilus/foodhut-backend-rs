pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        KitchenBlocked,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenBlocked => (
                    StatusCode::OK,
                    Json(json!({ "message": "Kitchen blocked successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToBlockKitchen,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToBlockKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to block kitchen" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
