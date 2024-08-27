use std::{io::Read, sync::Arc};

use crate::repository;
use axum::{
    async_trait,
    extract::{multipart::Field, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use axum_typed_multipart::{
    FieldData, TryFromField, TryFromMultipart, TypedMultipart, TypedMultipartError,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use serde::Deserialize;
use serde_json::json;
use tempfile::NamedTempFile;

use crate::{
    api::auth::middleware::AdminAuth,
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

async fn create_ad(
    State(ctx): State<Arc<Context>>,
    auth: AdminAuth,
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

    match repository::ad::create(
        ctx.db_conn.clone(),
        repository::ad::CreateAdPayload {
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
    auth: AdminAuth,
    pagination: Pagination,
    Query(filters): Query<repository::ad::Filters>,
) -> impl IntoResponse {
    match repository::ad::find_many(ctx.db_conn.clone(), pagination.clone(), filters).await {
        Ok(paginated_ads) => (StatusCode::OK, Json(json!(paginated_ads))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch ads"})),
        ),
    }
}

async fn get_ad_by_id(
    Path(id): Path<String>,
    auth: AdminAuth,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::ad::find_by_id(ctx.db_conn.clone(), id).await {
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

async fn update_ad_by_id(
    Path(id): Path<String>,
    State(ctx): State<Arc<Context>>,
    auth: AdminAuth,
    TypedMultipart(mut payload): TypedMultipart<UpdateAdPayload>,
) -> impl IntoResponse {
    let ad = match repository::ad::find_by_id(ctx.db_conn.clone(), id.clone()).await {
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

    match repository::ad::update_by_id(
        ctx.db_conn.clone(),
        id,
        repository::ad::UpdateAdPayload {
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

async fn delete_ad_by_id(
    Path(id): Path<String>,
    auth: AdminAuth,
    State(ctx): State<Arc<Context>>,
) -> impl IntoResponse {
    match repository::ad::find_by_id(ctx.db_conn.clone(), id.clone()).await {
        Ok(Some(ad)) => {
            if let Err(_) =
                utils::storage::delete_file(ctx.storage.clone(), ad.banner_image.clone()).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to delete ad" })),
                );
            }

            match repository::ad::delete_by_id(ctx.db_conn.clone(), id.clone()).await {
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
    Router::new()
        .route("/", post(create_ad).get(get_ads))
        .route(
            "/:id",
            get(get_ad_by_id)
                .patch(update_ad_by_id)
                .delete(delete_ad_by_id),
        )
}

// TODO: have a cron worker run nightly to delete stale/expired ads
