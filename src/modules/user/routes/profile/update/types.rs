pub mod request {
    use crate::modules::auth::middleware::Auth;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub email: Option<String>,
        pub phone_number: Option<String>,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
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
        UserUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::UserUpdated => (
                    StatusCode::OK,
                    Json(json!({ "message": "User updated successfully" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToUpdateUser,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to update user" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
