pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub phone_number: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        CheckPhoneForVerificationOtp,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CheckPhoneForVerificationOtp => (
                    StatusCode::OK,
                    Json(json!({"message": "Check your phone for a verification OTP"})),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchUser,
        UserNotFound,
        FailedToSendOtp,
        OtpNotExpired,
        UserAlreadyVerified,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::OtpNotExpired => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "OTP not expired"})),
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
                Error::UserAlreadyVerified => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "User already verified"
                    })),
                )
                    .into_response(),
                Error::FailedToSendOtp => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to send OTP"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
