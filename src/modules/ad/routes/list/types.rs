pub mod request {
    use crate::{modules::ad::repository, utils::pagination::Pagination};
    use serde::Deserialize;

    pub type Filters = repository::Filters;

    #[derive(Deserialize)]
    pub struct Payload {
        pub pagination: Pagination,
        pub filters: Filters,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::{modules::ad::repository::Ad, utils::pagination::Paginated};

    pub enum Success {
        PaginatedAds(Paginated<Ad>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::PaginatedAds(ads) => (StatusCode::OK, Json(json!(ads))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchAds,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchAds => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch ads"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
