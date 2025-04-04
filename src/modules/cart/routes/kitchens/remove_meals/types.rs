pub mod request {
    pub struct Payload {
        pub kitchen_id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        ItemsRemovedFromCart,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::ItemsRemovedFromCart => (
                    StatusCode::OK,
                    Json(json!({ "message": "Items removed from cart" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        InvalidDate(String),
        ImageUploadFailed,
        AdCreationFailed,
        CartNotFound,
        FailedToFindCart,
        NoItemsToCheckout,
        FailedToCheckoutCart,
        FailedToRemoveItemsFromCart,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToRemoveItemsFromCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to remove items from cart"})),
                )
                    .into_response(),
                Self::FailedToCheckoutCart => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to checkout cart" })),
                )
                    .into_response(),
                Self::NoItemsToCheckout => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "No items to checkout!" })),
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
                Self::InvalidDate(err) => {
                    (StatusCode::BAD_REQUEST, Json(json!({ "error": err}))).into_response()
                }
                Self::AdCreationFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Ad creation failed"})),
                )
                    .into_response(),
                Self::ImageUploadFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to upload image" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
