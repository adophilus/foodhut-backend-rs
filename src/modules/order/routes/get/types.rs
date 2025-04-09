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

    use crate::modules::order::repository::{FullOrder, FullOrderWithOwner};

    pub enum Success {
        Order(FullOrder),
        OrderWithOwner(FullOrderWithOwner),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Order(order) => (StatusCode::OK, Json(json!(order))).into_response(),
                Self::OrderWithOwner(order) => (StatusCode::OK, Json(json!(order))).into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchOrder,
        OrderNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToFetchOrder => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch order" })),
                )
                    .into_response(),
                Self::OrderNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Order not found" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
