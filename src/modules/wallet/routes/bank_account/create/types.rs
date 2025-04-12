pub mod request {
    use crate::modules::auth::middleware::Auth;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub bvn: String,
        pub bank_code: String,
        pub account_number: String,
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
        ApplicationSent,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::ApplicationSent => (
                    StatusCode::OK,
                    Json(json!({ "message": "Application sent" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateUser,
        FailedToSendApplication(Option<String>),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToSendApplication(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to send application" })),
                )
                    .into_response(),
                Self::FailedToUpdateUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update user" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
