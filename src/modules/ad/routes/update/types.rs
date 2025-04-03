pub mod request {
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart)]
    pub struct Body {
        pub link: Option<String>,
        pub duration: Option<i32>,
        #[form_data(limit = "10MiB")]
        pub banner_image: Option<FieldData<NamedTempFile>>,
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
        AdUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AdUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Ad updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToFetchAds,
        AdNotFound,
        FailedToFetchAd,
        FailedToUploadImage,
        FailedToUpdateAd,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateAd => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update ad" })),
                )
                    .into_response(),
                Self::FailedToUploadImage => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to upload image" })),
                )
                    .into_response(),
                Self::FailedToFetchAd => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to find ad"})),
                )
                    .into_response(),
                Self::AdNotFound => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Ad not found" })),
                )
                    .into_response(),
                Self::FailedToFetchAds => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to fetch ads"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
