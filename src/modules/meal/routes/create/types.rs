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
                        Price(
                            BigDecimal::from_f32(price).unwrap_or(BigDecimal::from_u8(0).unwrap()),
                        )
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
        pub name: String,
        pub description: String,
        pub price: Price,
        #[form_data(limit = "10MiB")]
        pub cover_image: FieldData<NamedTempFile>,
    }

    pub struct Payload {
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use crate::modules::meal::repository::Meal;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;
    use validator::ValidationErrors;

    pub enum Success {
        MealCreated(Meal),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::MealCreated(meal) => (
                    StatusCode::CREATED,
                    Json(json!({
                        "message": "Meal created!",
                        "id": meal.id
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToCreateMeal,
        KitchenNotCreated,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::KitchenNotCreated => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Kitchen not created" })),
                )
                    .into_response(),
                Self::FailedToCreateMeal => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create meal" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
