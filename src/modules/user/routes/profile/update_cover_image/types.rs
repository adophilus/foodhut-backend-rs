pub mod request {
    use crate::modules::auth::middleware::Auth;
    use axum_typed_multipart::{FieldData, TryFromMultipart};
    use tempfile::NamedTempFile;

    #[derive(TryFromMultipart)]
    pub struct Body {
        pub profile_picture: FieldData<NamedTempFile>,
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
        ProfilePictureUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::ProfilePictureUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "Profile picture updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateProfilePicture,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateProfilePicture => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update profile picture" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
