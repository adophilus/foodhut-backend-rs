pub mod request {
    use crate::modules::{auth::middleware::Auth, order::repository};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub status: repository::OrderStatus,
        pub as_kitchen: Option<bool>,
    }

    pub struct Payload {
        pub id: String,
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        OrderStatusUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::OrderStatusUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Order status updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateOrderStatus,
        OrderNotFound,
        KitchenNotOwner,
        UserNotOwnKitchen,
        UserNotOwner,
        InvalidStatusTransitionForKitchen,
        InvalidStatusTransitionForUser,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UserNotOwner => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "message": "User does not own this order" })),
                )
                    .into_response(),
                Self::InvalidStatusTransitionForUser => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "message": "Invalid status transition for user" })),
                )
                    .into_response(),
                Self::InvalidStatusTransitionForKitchen => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "message": "Invalid status transition for kitchen" })),
                )
                    .into_response(),
                Self::UserNotOwnKitchen => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "message": "User does not own a kitchen" })),
                )
                    .into_response(),
                Self::KitchenNotOwner => (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "message": "Kitchen does not own this order" })),
                )
                    .into_response(),
                Self::FailedToUpdateOrderStatus => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to updae order status" })),
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
