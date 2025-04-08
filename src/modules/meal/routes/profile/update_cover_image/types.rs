pub mod request {
    use crate::modules::auth::middleware::Auth;
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart)]
    pub struct Body {
        pub cover_image: FieldData<NamedTempFile>,
    }

    pub struct Payload {
        pub body: Body,
        pub auth: Auth,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        CoverImageUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CoverImageUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Cover image updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateCoverImage,
        FailedToFetchKitchen,
        KitchenNotFound,
        NotKitchenOwner,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateCoverImage => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update cover image" })),
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
