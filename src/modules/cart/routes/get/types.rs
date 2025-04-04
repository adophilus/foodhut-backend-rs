pub mod request {}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde::Serialize;
    use serde_json::json;

    use crate::modules::{kitchen::repository::Kitchen, meal::repository::MealWithCartStatus};

    #[derive(Debug, Serialize)]
    pub struct MealWithQuantity {
        pub meal: MealWithCartStatus,
        pub quantity: i32,
    }

    #[derive(Debug, Serialize)]
    pub struct KitchenWithMeals {
        pub kitchen: Kitchen,
        pub meals: Vec<MealWithQuantity>,
    }

    pub enum Success {
        Items(Vec<KitchenWithMeals>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Items(items) => (StatusCode::OK, Json(json!(items))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchActiveCart,
        CartNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchActiveCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch active cart" })),
                )
                    .into_response(),
                Self::CartNotFound => (StatusCode::OK, Json(json!([]))).into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
