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
        MealDeleted,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealDeleted => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal deleted successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToDeleteMeal,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToDeleteMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
