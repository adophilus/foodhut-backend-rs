use axum::{http::StatusCode, Json};
use serde_json::json;
use validator::ValidationErrors;

pub fn into_response(errors: ValidationErrors) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({"errors": errors})))
}
