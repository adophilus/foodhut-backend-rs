use super::service;
use crate::modules::user;
use crate::modules::user::repository::User;
use crate::types::Context;
use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::RequestPartsExt;
use axum::{async_trait, Json};
use axum::{extract::Extension, http, http::request::Parts, response::Response};
use serde::Serialize;
use serde_json::json;
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

async fn get_user_from_header(ctx: Arc<Context>, header: String) -> Result<User, Error> {
    let session_id = get_session_id_from_header(header)?;
    let session = service::auth::verify_access_token(ctx.clone(), session_id)
        .await
        .map_err(|_| Error::InvalidSession)?;

    user::repository::find_by_id(&ctx.db_conn.pool, session.user_id)
        .await
        .map_err(|_| Error::InvalidSession)?
        .ok_or(Error::InvalidSession)
        .map(|user| {
            if user.is_deleted {
                return Err(Error::InvalidSession);
            }

            Ok(user)
        })?
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

async fn get_user_from_request<State: Send + Sync>(
    ctx: Arc<Context>,
    parts: &mut Parts,
    _: &State,
) -> Result<User, Response> {
    let headers = parts.extract::<HeaderMap>().await.unwrap();

    let err = (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "Invalid session token"})),
    );

    let auth_header = headers
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(err.clone().into_response())?;

    get_user_from_header(ctx.clone(), auth_header.to_string())
        .await
        .map_err(|_| err.clone().into_response())
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(ctx) = parts.extract::<Extension<Arc<Context>>>().await.unwrap();
        get_user_from_request(ctx, parts, state)
            .await
            .map(|user| Self { user })
    }
}

#[derive(Serialize, Clone)]
pub struct AdminAuth {
    pub user: User,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AdminAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(ctx) = parts.extract::<Extension<Arc<Context>>>().await.unwrap();

        let user = get_user_from_request(ctx, parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Unauthorized" })),
                )
                    .into_response()
            })?;

        if !user::repository::is_admin(&user) {
            return Err(
                (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))).into_response(),
            );
        }

        Ok(Self { user })
    }
}
