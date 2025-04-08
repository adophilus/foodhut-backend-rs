pub mod request {
    use crate::{modules::auth::middleware::Auth, utils::pagination::Pagination};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Filters {
        pub kitchen_id: Option<String>,
        pub search: Option<String>,
        pub is_liked: Option<bool>,
        pub as_kitchen: Option<bool>,
    }

    pub struct Payload {
        pub auth: Option<Auth>,
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
        FailedToFetchMeals,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchMeals => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch meals" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
