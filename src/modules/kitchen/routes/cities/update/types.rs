pub mod request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Body {
        pub name: Option<String>,
        pub state: Option<String>,
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
        CityUpdated,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::CityUpdated => {
                    (StatusCode::OK, Json(json!({ "message": "City updated"}))).into_response()
                }
            }
        }
    }

    pub enum Error {
        FailedToUpdateCity,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToUpdateCity => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update city" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
