pub mod request {
    pub struct Payload {
        pub id: String,
    }
}

pub mod response {
    use axum::{http::StatusCode, response::IntoResponse};

    pub enum Success {
        Media(String),
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::Media(id) => (StatusCode::OK, id).into_response(),
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
