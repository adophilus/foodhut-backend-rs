pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::ad::repository::Ad;

    pub enum Success {
        Ad(Ad),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Ad(ad) => (StatusCode::OK, Json(json!(ad))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchAds,
        AdNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AdNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Ad not found" })),
                )
                    .into_response(),
                Self::FailedToFetchAds => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch ads"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
