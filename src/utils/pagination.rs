use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub meta: PaginatedMeta,
}

#[derive(Serialize, Clone)]
pub struct PaginatedMeta {
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

impl<T: Clone> Paginated<T> {
    pub fn new(items: Vec<T>, total: u32, page: u32, per_page: u32) -> Paginated<T> {
        Self {
            items,
            meta: PaginatedMeta {
                total,
                page,
                per_page,
            },
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    10
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Pagination {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.extract::<Query<Pagination>>().await {
            Ok(Query(pagination)) => Ok(pagination),
            _ => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid pagination options"})),
            )
                .into_response()),
        }
    }
}
