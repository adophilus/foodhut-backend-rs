pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Payload {
        pub name: String,
        pub state: String,
    }
}

pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        CityCreated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CityCreated => {
                    (StatusCode::OK, Json(json!({ "message": "City created" }))).into_response()
                }
            }
        }
    }

    pub enum Error {
        FailedToCreateCity,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToCreateCity => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to create city"})),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
