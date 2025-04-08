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
    pub struct Body {
        pub name: Option<String>,
        pub address: Option<String>,
        pub phone_number: Option<String>,
        #[validate(custom(function = "validate_kitchen_type"))]
        #[serde(rename = "type")]
        pub r#type: Option<String>,
        #[validate(custom(function = "validate_opening_time"))]
        pub opening_time: Option<String>,
        #[validate(custom(function = "validate_closing_time"))]
        pub closing_time: Option<String>,
        pub preparation_time: Option<String>,
        pub delivery_time: Option<String>,
        pub is_available: Option<bool>,
    }

    pub struct Payload {
        pub id: String,
        pub body: Body,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        KitchenUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Kitchen updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateKitchen,
        FailedToFetchKitchen,
        KitchenNotFound,
        NotKitchenOwner,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update kitchen" })),
                )
                    .into_response(),
                Self::NotKitchenOwner => (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "You are not the owner of this kitchen"})),
                )
                    .into_response(),
                Self::FailedToFetchKitchen => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to fetch kitchen" })),
                )
                    .into_response(),
                Self::KitchenNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Kitchen not found" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
