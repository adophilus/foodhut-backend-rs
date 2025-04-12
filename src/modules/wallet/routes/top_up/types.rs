pub mod request {
    pub use crate::modules::auth::middleware::Auth;
    use bigdecimal::BigDecimal;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub amount: BigDecimal,
    }

    pub struct Payload {
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        TopupInvoiceLink(String),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::TopupInvoiceLink(link) => {
                    (StatusCode::OK, Json(json!({ "url": link }))).into_response()
                }
            }
        }
    }

    pub enum Error {
        FailedToCreateTopupInvoiceLink,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToCreateTopupInvoiceLink => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create topup invoice link" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
