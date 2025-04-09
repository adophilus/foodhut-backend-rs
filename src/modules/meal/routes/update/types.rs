pub mod request {
    use crate::modules::auth::middleware::Auth;
    use async_trait::async_trait;
    use axum::extract::multipart::Field;
    use axum_typed_multipart::{FieldData, TryFromField, TryFromMultipart, TypedMultipartError};
    use bigdecimal::{BigDecimal, FromPrimitive};
    use tempfile::NamedTempFile;

    #[derive(Debug, Clone)]
    pub struct Price(pub BigDecimal);

    #[async_trait]
    impl TryFromField for Price {
        async fn try_from_field<'a>(
            field: Field<'a>,
            _: Option<usize>,
        ) -> Result<Self, TypedMultipartError> {
            field
                .text()
                .await
                .map(|text| {
                    text.parse::<f32>().map(|price| {
                        Price(BigDecimal::from_f32(price).unwrap_or(BigDecimal::from(0)))
                    })
                })
                .map_err(|err| {
                    tracing::error!("Error occurred while parsing body: {}", err);
                    TypedMultipartError::InvalidRequestBody { source: err }
                })
                .unwrap()
                .map_err(|err| {
                    tracing::error!("Error occurred while parsing body: {}", err);
                    TypedMultipartError::UnknownField {
                        field_name: String::from("price"),
                    }
                })
        }
    }

    #[derive(TryFromMultipart)]
    pub struct Body {
        pub name: Option<String>,
        pub description: Option<String>,
        pub price: Option<Price>,
        pub is_available: Option<bool>,
        #[form_data(limit = "10MiB")]
        pub cover_image: Option<FieldData<NamedTempFile>>,
    }

    pub struct Payload {
        pub id: String,
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        MealUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Meal updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        MealNotFound,
        FailedToUpdateMeal,
        NotMealOwner,
        KitchenNotCreated,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::NotMealOwner => (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "You are not the owner of this meal"})),
                )
                    .into_response(),
                Self::MealNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Meal not found"})),
                )
                    .into_response(),
                Self::KitchenNotCreated => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Kitchen not created" })),
                )
                    .into_response(),
                Self::FailedToUpdateMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to update meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
