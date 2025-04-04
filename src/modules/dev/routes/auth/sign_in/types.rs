pub mod request {
    use crate::modules::order::repository::PaymentMethod;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub phone_number: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::auth::repository::session::Session;

    pub enum Success {
        Tokens(Session),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Tokens(session) => (
                    StatusCode::OK,
                    Json(json!({
                        "access_token": session.access_token,
                        "refresh_token": session.refresh_token,
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToCreateSession,
        FailedToFetchUser,
        UserNotFound,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToCreateSession => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create session" })),
                )
                    .into_response(),
                Self::FailedToFetchUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch user" })),
                )
                    .into_response(),
                Self::UserNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "User not found"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
