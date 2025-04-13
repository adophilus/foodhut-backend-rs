pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{
        extract::Json,
        http::StatusCode,
        response::{IntoResponse, Redirect},
    };
    use serde_json::json;

    pub enum Success {
        UserRegistered,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UserRegistered => (
                    StatusCode::OK,
                    Json(json!({ "message": "User registered" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        UserNotFound,
        FailedToRegisterUser,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UserNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "User not found" })),
                )
                    .into_response(),
                Self::FailedToRegisterUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to register user" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
