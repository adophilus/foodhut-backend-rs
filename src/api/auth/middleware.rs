use axum::extract::{FromRequestParts, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{async_trait, Json};
use axum::{
    extract::{Extension, Request},
    http,
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use serde_json::json;

use crate::repository;
use crate::repository::user::User;
use crate::types::Context;
use crate::utils::database::DatabaseConnection;
use std::sync::Arc;

enum Error {
    InvalidSession,
}

fn get_session_id_from_header(header: String) -> Result<String, Error> {
    header
        .split(" ")
        .skip(1)
        .next()
        .map(|h| h.to_string())
        .ok_or(Error::InvalidSession)
}

async fn get_user_from_header(
    db_conn: DatabaseConnection,
    header: String,
) -> Result<repository::user::User, Error> {
    let session_id = get_session_id_from_header(header)?;
    match repository::session::find_by_id(db_conn.clone(), session_id).await {
        Some(session) => match repository::user::find_by_id(db_conn.clone(), session.user_id).await
        {
            Some(user) => Ok(user),
            None => Err(Error::InvalidSession),
        },
        None => Err(Error::InvalidSession),
    }
}

// pub async fn auth(
//     req: Request,
//     State(ctx): State<Context>,
//     next: Next,
// ) -> Result<Response, ApiResponse<&'static str, &'static str>> {
//     match req
//         .headers()
//         .get(http::header::AUTHORIZATION)
//         .and_then(|header| header.to_str().ok())
//     {
//         Some(auth_header) => {
//             match get_user_from_header(ctx.db_conn.clone(), auth_header.to_string()).await {
//                 Ok(user) => Ok(next.run(req).await),
//                 Err(_) => Err(ApiResponse::err("Invalid session token")),
//             }
//         }
//         None => Err(ApiResponse::err("Invalid session token")),
//     }
// }

#[derive(Serialize, Clone)]
pub struct Auth {
    pub user: User,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(ctx) = parts.extract::<Extension<Arc<Context>>>().await.unwrap();
        let headers = parts.extract::<HeaderMap>().await.unwrap();

        let err = (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session token"})),
        );

        let auth_header = headers
            .get(http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(err.clone().into_response())?;

        get_user_from_header(ctx.db_conn.clone(), auth_header.to_string())
            .await
            .map(|user| Self { user })
            .map_err(|_| err.clone().into_response())
    }
}

#[derive(Serialize, Clone)]
pub struct AdminAuth {
    pub user: User,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AdminAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(auth) = parts.extract::<Extension<Auth>>().await.unwrap();
        let headers = parts.extract::<HeaderMap>().await.unwrap();

        if !repository::user::is_admin(&auth.user) {
            let err = (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid session token"})),
            );
        }

        Ok(Self { user: auth.user })
    }
}
