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
        MealUnliked,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealUnliked => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal unliked successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUnlikeMeal,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUnlikeMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to unlike meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
