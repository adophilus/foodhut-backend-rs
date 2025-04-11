pub mod request {
    pub use super::super::super::super::repository::FindManyMealsAndKitchenFilters as Filters;
    pub use crate::utils::pagination::Pagination;

    pub struct Payload {
        pub pagination: Pagination,
        pub filters: Filters,
    }
}

pub mod response {
    pub use super::super::super::super::repository::DatabasePaginatedMealOrKitchen;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Result(DatabasePaginatedMealOrKitchen),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Result(res) => (StatusCode::OK, Json(json!(res))).into_response(),
            }
        }
    }

    pub enum Error {
        SearchFailed,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::SearchFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Search failed" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
