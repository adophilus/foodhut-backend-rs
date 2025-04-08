pub mod request {
    use crate::modules::auth::middleware::Auth;

    pub struct Payload {
        pub id: String,
        pub auth: Option<Auth>,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::{modules::meal::repository::MealWithCartStatus, utils::pagination::Paginated};

    pub enum Success {
        Meal(MealWithCartStatus),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Meal(meal) => (StatusCode::OK, Json(json!(meal))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchMeal,
        MealNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Meal not found" })),
                )
                    .into_response(),
                Self::FailedToFetchMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
