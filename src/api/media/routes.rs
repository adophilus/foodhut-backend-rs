use crate::types::Context;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tempfile::NamedTempFile;
use ulid::Ulid;

async fn get_media(Path(id): Path<String>) -> impl IntoResponse {
    return (StatusCode::OK, "hi");
}

#[derive(TryFromMultipart)]
struct UploadPayload {
    #[form_data(limit = "10MiB")]
    file: FieldData<NamedTempFile>,
}

async fn upload_media(
    State(ctx): State<Arc<Context>>,
    TypedMultipart(payload): TypedMultipart<UploadPayload>,
) -> impl IntoResponse {
    let file_name = Ulid::new().to_string();
    let file_path = format!("public/uploads/{}", file_name);
    if let Err(err) = payload.file.contents.persist(file_path.clone()) {
        tracing::error!("Failed to save uploaded file: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload file" })),
        );
    }
    return (
        StatusCode::OK,
        Json(json!({
            "public_id": file_name,
            "signature": file_name,
            "secure_url": format!("{}/api/media/{}", ctx.app.url, file_name)
        })),
    );
}

async fn delete_media(Path(id): Path<String>) -> impl IntoResponse {
    match std::fs::remove_file(format!("public/uploads/{}", id)) {
        Ok(_) => (StatusCode::OK, Json(json!({ "message": "Deleted file" }))),
        Err(err) => {
            tracing::debug!("Failed to delete uploaded file {}: {:?}", id, err);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Failed to delete file"})),
            )
        }
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new()
        .route("/:id", get(get_media))
        .route("/upload", post(upload_media))
        .route("/destroy", delete(delete_media))
}
