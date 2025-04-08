pub mod request {
    use crate::{modules::kitchen::repository, utils::pagination::Pagination};

    pub type Filters = repository::FindManyFilters;

    pub struct Payload {
        pub filters: Filters,
        pub pagination: Pagination,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::{modules::kitchen::repository::Kitchen, utils::pagination::Paginated};

    pub enum Success {
        Kitchens(Paginated<Kitchen>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Kitchens(kitchens) => (StatusCode::OK, Json(json!(kitchens))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchKitchens,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchKitchens => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch kitchens" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
