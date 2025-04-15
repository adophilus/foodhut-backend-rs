pub mod request {
    pub use crate::modules::auth::middleware::Auth;
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::kitchen::repository::KitchenCity;

    pub enum Success {
        Cities(Vec<KitchenCity>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Cities(cities) => (StatusCode::OK, Json(json!(cities))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchCities,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchCities => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch cities" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
