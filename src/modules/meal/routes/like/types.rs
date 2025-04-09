pub mod request {
    use crate::modules::auth::middleware::Auth;

    pub struct Payload {
        pub id: String,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        MealLiked,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealLiked => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal liked successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToLikeMeal,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToLikeMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to like meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
