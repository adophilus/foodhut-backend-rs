pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        Types(Vec<String>),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Types(types) => (StatusCode::OK, Json(json!(types))).into_response(),
            }
        }
    }

    pub enum Error {}

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            "".into_response()
        }
    }

    pub type Response = Result<Success, Error>;
}
