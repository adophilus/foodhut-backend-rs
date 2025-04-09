pub mod request {
    use crate::{
        modules::{auth::middleware::Auth, order::repository::OrderSimpleStatus},
        utils::pagination::Pagination,
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Filters {
        pub status: Option<OrderSimpleStatus>,
        pub kitchen_id: Option<String>,
    }

    pub struct Payload {
        pub filters: Filters,
        pub pagination: Pagination,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::{modules::order::repository::FullOrder, utils::pagination::Paginated};

    pub enum Success {
        Orders(Paginated<FullOrder>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Orders(orders) => (StatusCode::OK, Json(json!(orders))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchOrders,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchOrders => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch orders" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
