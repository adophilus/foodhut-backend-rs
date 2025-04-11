pub mod request {
    pub use crate::modules::auth::middleware::Auth;

    pub struct Payload {
        pub auth: Auth,
    }
}

pub mod response {
    use super::super::super::super::super::repository::User;
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        User(User),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::User(user) => (StatusCode::OK, Json(json!(user))).into_response(),
            }
        }
    }

    pub enum Error {}

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            unreachable!()
        }
    }

    pub type Response = Result<Success, Error>;
}
