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
