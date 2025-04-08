pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::kitchen::repository::Kitchen;

    pub enum Success {
        Kitchen(Kitchen),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Kitchen(kitchen) => (StatusCode::OK, Json(json!(kitchen))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchKitchen,
        KitchenNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch kitchen" })),
                )
                    .into_response(),
                Self::KitchenNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Kitchen not found" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
