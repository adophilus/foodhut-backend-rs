pub mod request {
    pub struct Payload {
        pub meal_id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        MealRemovedFromCart,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealRemovedFromCart => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal removed from cart" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        MealNotFound,
        CartNotFound,
        FailedToFindCart,
        FailedToFetchMeal,
        FailedToRemoveMealFromCart,
        MealNotFoundInCart,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToRemoveMealFromCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to remove meal from cart" })),
                )
                    .into_response(),
                Self::MealNotFoundInCart => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Meal not found in cart" })),
                )
                    .into_response(),
                Self::FailedToFetchMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch meal" })),
                )
                    .into_response(),
                Self::MealNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Meal not found" })),
                )
                    .into_response(),
                Self::FailedToFindCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to find cart"})),
                )
                    .into_response(),
                Self::CartNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Cart not found"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
