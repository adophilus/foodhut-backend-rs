pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub quantity: i32,
    }

    pub struct Payload {
        pub meal_id: String,
        pub body: Body,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        CartUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CartUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Cart updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        MealNotFound,
        FailedToFetchMeal,
        FailedToUpdateCart,
        FailedToSetItemInCart,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update cart" })),
                )
                    .into_response(),
                Self::FailedToSetItemInCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to set item in cart"})),
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
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
