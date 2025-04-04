pub mod response {
    use axum::{extract::Json, http::StatusCode, response::IntoResponse};
    use serde_json::json;

    pub enum Success {
        PushNotificationSent,
    }

    impl IntoResponse for Success {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::PushNotificationSent => (
                    StatusCode::OK,
                    Json(json!({ "message": "Push notification sent" })),
                )
                    .into_response(),
            }
        }
    }

    pub enum Error {
        FailedToSendPushNotification,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> axum::response::Response {
            match self {
                Self::FailedToSendPushNotification => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to send push notification" })),
                )
                    .into_response(),
            }
        }
    }

    pub type Response = Result<Success, Error>;
}
