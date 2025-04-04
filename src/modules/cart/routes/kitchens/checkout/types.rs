pub mod request {
    use crate::modules::order::repository::PaymentMethod;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub payment_method: PaymentMethod,
        pub delivery_address: String,
        pub delivery_date: Option<u64>,
        pub dispatch_rider_note: String,
    }

    #[derive(Deserialize)]
    pub struct Payload {
        pub kitchen_id: String,
        pub body: Body,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::order::repository::Order;

    pub enum Success {
        CheckoutSuccessful(Order),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CheckoutSuccessful(order) => (
                    StatusCode::CREATED,
                    Json(json!({ "message": "Cart checkedout successfully", "id": order.id })),
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
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
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
