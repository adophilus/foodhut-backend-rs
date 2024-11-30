use std::{io::Read, sync::Arc};

use super::repository;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde_json::json;
use tempfile::NamedTempFile;

use crate::{
    modules::auth::middleware::AdminAuth,
    types::Context,
    utils::{self, pagination::Pagination},
};

#[derive(TryFromMultipart)]
pub struct CreateAdPayload {
    link: String,
    duration: i32,
    #[form_data(limit = "10MiB")]
    banner_image: FieldData<NamedTempFile>,
}

async fn create(
    State(ctx): State<Arc<Context>>,
    _: AdminAuth,
    TypedMultipart(mut payload): TypedMultipart<CreateAdPayload>,
) -> impl IntoResponse {
    let mut buf: Vec<u8> = vec![];

    if let Err(err) = payload.banner_image.contents.read_to_end(&mut buf) {
        tracing::error!("Failed to read the uploaded file {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to upload image" })),
        );
    }

    let banner_image = match utils::storage::upload_file(ctx.storage.clone(), buf).await {
        Ok(media) => media,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to upload image" })),
            );
        }
    };

    match repository::create(
        &ctx.db_conn.pool,
        repository::CreateAdPayload {
            banner_image,
            link: payload.link,
            duration: payload.duration,
        },
    )
    .await
    {
        Ok(ad) => (
            StatusCode::CREATED,
            Json(json!({
                "message": "Ad created!",
                "id": ad.id
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Ad creation failed"})),
        ),
    }
}

async fn get_ads(
    State(ctx): State<Arc<Context>>,
    pagination: Pagination,
    Query(filters): Query<repository::Filters>,
) -> impl IntoResponse {
    match repository::find_many(&ctx.db_conn.pool, pagination.clone(), filters).await {
        Ok(paginated_ads) => (StatusCode::OK, Json(json!(paginated_ads))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch ads"})),
        ),
    }
}

async fn get_by_id(Path(id): Path<String>, State(ctx): State<Arc<Context>>) -> impl IntoResponse {
    match repository::find_by_id(&ctx.db_conn.pool, id).await {
        Ok(Some(ad)) => (StatusCode::OK, Json(json!(ad))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Ad not found" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch ads"})),
        ),
    }
}

#[derive(TryFromMultipart)]
pub struct UpdateAdPayload {
    pub link: Option<String>,
    pub duration: Option<i32>,
    #[form_data(limit = "10MiB")]
    banner_image: Option<FieldData<NamedTempFile>>,
}

async fn update_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    _: AdminAuth,
    TypedMultipart(payload): TypedMultipart<UpdateAdPayload>,
) -> impl IntoResponse {
    let ad = match repository::find_by_id(&ctx.db_conn.pool, id.clone()).await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to find ad"})),
            )
        }
        Ok(Some(kitchen)) => kitchen,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Ad not found"})),
            )
        }
    };

    let banner_image = match payload.banner_image {
        Some(mut banner_image) => {
            let mut buf: Vec<u8> = vec![];

            if let Err(err) = banner_image.contents.read_to_end(&mut buf) {
                tracing::error!("Failed to read the uploaded file {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to upload image" })),
                );
            }

            let media = match utils::storage::update_file(
                ctx.storage.clone(),
                ad.banner_image.clone(),
                buf,
            )
            .await
            {
                Ok(media) => media,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to upload image" })),
                    );
                }
            };

            Some(media)
        }
        None => None,
    };

    match repository::update_by_id(
        &ctx.db_conn.pool,
        id,
        repository::UpdateAdPayload {
            banner_image,
            link: payload.link,
            duration: payload.duration,
        },
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Ad updated successfully" })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to update ad" })),
        ),
    }
}

async fn delete_by_id(
    Path(id): Path<String>,
    _: AdminAuth,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::find_by_id(&ctx.db_conn.pool, id.clone()).await {
        Ok(Some(ad)) => {
            if let Err(_) =
                utils::storage::delete_file(ctx.storage.clone(), ad.banner_image.clone()).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete ad" })),
                );
            }

            match repository::delete_by_id(&ctx.db_conn.pool, id.clone()).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({ "message": "Ad deleted successfully" })),
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "message": "Failed to delete ad" })),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Ad not found"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to delete ad" })),
        ),
    }
}

pub fn get_router() -> Router<Arc<Context>> {
    Router::new().route("/", post(create).get(get_ads)).route(
        "/:id",
        get(get_by_id).patch(update_by_id).delete(delete_by_id),
    )
}
