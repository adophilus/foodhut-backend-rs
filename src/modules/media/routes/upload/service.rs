use super::types::{request, response};
use crate::types::Context;
use std::sync::Arc;
use ulid::Ulid;

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    let file_name = Ulid::new().to_string();
    let file_path = format!("public/uploads/{}", file_name);

    payload
        .file
        .contents
        .persist(file_path.clone())
        .map_err(|err| {
            tracing::error!("Failed to save uploaded file: {:?}", err);
            response::Error::FailedToUploadMedia
        })
        .map(|_| response::Success::UploadedMedia(ctx.app.url.clone(), file_name))
}
