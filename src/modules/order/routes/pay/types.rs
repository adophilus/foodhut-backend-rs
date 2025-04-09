pub mod request {
    use crate::modules::kitchen::types;
    use regex::Regex;
    use serde::Deserialize;
    use std::borrow::Cow;
    use validator::{Validate, ValidationError};

    fn validate_kitchen_type(r#type: &str) -> Result<(), ValidationError> {
        match types::KITCHEN_TYPES.contains(&r#type) {
            true => Ok(()),
            false => Err(ValidationError::new("INVALID_KITCHEN_TYPE")
                .with_message(Cow::from("Invalid kitchen type"))),
        }
    }

    fn validate_opening_time(time_str: &str) -> Result<(), ValidationError> {
        let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid opening time regex");
        match regex.is_match(time_str) {
            true => Ok(()),
            false => Err(
                ValidationError::new("INVALID_OPENING_TIME").with_message(Cow::from(
                    r"Opening time must be in 24 hour format (e.g: 08:00)",
                )),
            ),
        }
    }

    fn validate_closing_time(time_str: &str) -> Result<(), ValidationError> {
        let regex = Regex::new(r"^\d{2}:\d{2}$").expect("Invalid closing time regex");
        match regex.is_match(time_str) {
            true => Ok(()),
            false => Err(
                ValidationError::new("INVALID_CLOSING_TIME").with_message(Cow::from(
                    r"Closing time must be in 24 hour format (e.g: 20:00)",
                )),
            ),
        }
    }

    #[derive(Deserialize, Validate)]
    pub struct Payload {
        pub name: String,
        pub address: String,
        pub phone_number: String,
        #[validate(custom(function = "validate_kitchen_type"))]
        #[serde(rename = "type")]
        pub type_: String,
        #[validate(custom(code = "INVALID_OPENING_TIME", function = "validate_opening_time"))]
        pub opening_time: String,
        #[validate(custom(code = "INVALID_CLOSING_TIME", function = "validate_closing_time"))]
        pub closing_time: String,
        pub preparation_time: String,
        pub delivery_time: String,
        pub city_id: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;
    use validator::ValidationErrors;

    pub enum Success {
        KitchenCreated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenCreated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Kitchen created successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        AlreadyCreatedKitchen,
        FailedToCreateKitchen,
        FailedToValidate(ValidationErrors),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AlreadyCreatedKitchen => (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "You've already created a kitchen" })),
                )
                    .into_response(),
                Self::FailedToCreateKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create kitchen" })),
                )
                    .into_response(),
                Error::FailedToValidate(errors) => {
                    (StatusCode::BAD_REQUEST, Json(json!({ "errors": errors }))).into_response()
                }
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
