pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub phone_number: String,
        pub otp: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Tokens((String, String)),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Tokens((access_token, refresh_token)) => (
                    StatusCode::OK,
                    Json(json!({
                        "access_token": access_token,
                        "refresh_token": refresh_token,
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchUser,
        UserNotFound,
        FailedToCreateSession,
        InvalidOrExpiredOtp,
        OtpVerificationFailed,
        UnexpectedError,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToCreateSession => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create session" })),
                )
                    .into_response(),
                Error::FailedToFetchUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch user" })),
                )
                    .into_response(),
                Error::UserNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "User not found"})),
                )
                    .into_response(),
                Error::OtpVerificationFailed => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error" : "Failed to verify OTP"})),
                )
                    .into_response(),
                Error::InvalidOrExpiredOtp => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error" : "Invalid or expired OTP"})),
                )
                    .into_response(),
                Error::UnexpectedError => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Sorry an error occurred" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
