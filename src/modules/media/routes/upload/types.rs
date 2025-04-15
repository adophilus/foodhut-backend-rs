pub mod request {
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart)]
    pub struct Payload {
        #[form_data(limit = "10MiB")]
        pub file: FieldData<NamedTempFile>,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        UploadedMedia(String, String),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UploadedMedia(url, file_name) => (
                    StatusCode::OK,
                    Json(json!({
                        "public_id": file_name,
                        "signature": file_name,
                        "secure_url": format!("{}/api/media/{}", url, file_name)
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUploadMedia,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUploadMedia => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to upload media" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
