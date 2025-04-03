pub mod request {
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart)]
    pub struct Payload {
        pub link: String,
        pub duration: i32,
        #[form_data(limit = "10MiB")]
        pub banner_image: FieldData<NamedTempFile>,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    use crate::modules::ad::repository::Ad;

    pub enum Success {
        AdCreated(Ad),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AdCreated(ad) => (
                    StatusCode::CREATED,
                    Json(json!({
                        "message": "Ad created!",
                        "id": ad.id
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        ImageUploadFailed,
        AdCreationFailed,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::AdCreationFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Ad creation failed"})),
                )
                    .into_response(),
                Self::ImageUploadFailed => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to upload image" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
