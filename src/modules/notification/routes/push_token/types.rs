pub mod request {
    use crate::modules::auth::middleware::Auth;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub token: String,
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
        PushTokenCreated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::PushTokenCreated => (
                    StatusCode::CREATED,
                    Json(json!({
                        "message": "Push token created created!",
                    })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToCreatePushToken,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToCreatePushToken => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Push token creation failed"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
