pub mod request {
    use std::borrow::Cow;

    use regex::Regex;
    use serde::Deserialize;
    use validator::{Validate, ValidationError};

    fn validate_phone_number(phone_number: &str) -> Result<(), ValidationError> {
        let regex = Regex::new(r"^\+234\d{10}$").expect("Invalid phone number regex");
        match regex.is_match(phone_number) {
            true => Ok(()),
            false => Err(
                ValidationError::new("INVALID_CLOSING_TIME").with_message(Cow::from(
                    r"Phone number must be a nigerian phone number in international format (e.g: +234...)",
                )),
            ),
        }
    }

    #[derive(Deserialize, Validate)]
    pub struct Payload {
        #[validate(email(code = "INVALID_USER_EMAIL", message = "Invalid email address"))]
        pub email: String,
        #[validate(custom(code = "INVALID_PHONE_NUMBER", function = "validate_phone_number"))]
        pub phone_number: String,
        pub first_name: String,
        pub last_name: String,
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
        UserNotVerified,
        FailedToSendOtp,
        FailedToValidatePayload,
        OtpNotExpired,
        SignupFailed,
        EmailAlreadyInUse,
        PhoneNumberAlreadyInUse,
        FailedToCreateWallet,
        UnexpectedError,
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
                Error::FailedToValidatePayload => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Failed to validate payload"})),
                )
                    .into_response(),
                Error::UserNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "User not found"})),
                )
                    .into_response(),
                Error::UserNotVerified => (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "User not verified"})),
                )
                    .into_response(),
                Error::SignupFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Sign up failed!"
                    })),
                )
                    .into_response(),
                Error::EmailAlreadyInUse => (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "Email already in use" })),
                )
                    .into_response(),
                Error::PhoneNumberAlreadyInUse => (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "Phone number already in use" })),
                )
                    .into_response(),
                Error::FailedToSendOtp => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to send OTP"})),
                )
                    .into_response(),
                Error::FailedToCreateWallet => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create wallet" })),
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
