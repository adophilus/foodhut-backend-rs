use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use crate::database::DatabaseConnection;

#[derive(Clone)]
pub struct Context {
    pub db_conn: DatabaseConnection,
}

#[derive(Serialize, Clone)]
pub struct OkResponse<T: Serialize> {
    data: T,
}

#[derive(Serialize, Clone)]
pub struct ErrResponse<E: Serialize> {
    error: E,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum ApiResponse<T: Serialize, E: Serialize> {
    Ok(OkResponse<T>),
    Err(ErrResponse<E>),
}

impl<T: Serialize, E: Serialize> ApiResponse<T, E> {
    pub fn ok(data: T) -> ApiResponse<T, E> {
        ApiResponse::Ok(OkResponse { data })
    }

    pub fn err(error: E) -> ApiResponse<T, E> {
        ApiResponse::Err(ErrResponse { error })
    }
}

impl<T: Serialize, E: Serialize> IntoResponse for ApiResponse<T, E> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResponse::Ok(res) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&res).unwrap(),
            )
                .into_response(),
            ApiResponse::Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&err).unwrap(),
            )
                .into_response(),
        }
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    10
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Pagination {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
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
