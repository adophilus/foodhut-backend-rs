pub mod request {
    use crate::modules::{auth::middleware::Auth, order::repository::PaymentMethod};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub with: PaymentMethod,
    }

    pub struct Payload {
        pub id: String,
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use crate::modules::payment::service::PaymentDetails;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        PaymentDetails(PaymentDetails),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::PaymentDetails(details) => {
                    (StatusCode::OK, Json(json!(details))).into_response()
                }
            }
        }
    }

    pub enum Error {
        OrderNotFound,
        FailedToInitiateOrderPayment,
        PaymentAlreadyMade,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::PaymentAlreadyMade => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Payment has already been made" })),
                )
                    .into_response(),
                Self::OrderNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Order not found" })),
                )
                    .into_response(),
                Self::FailedToInitiateOrderPayment => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to initiate order payment" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
