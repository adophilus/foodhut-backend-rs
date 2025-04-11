use super::{
    service::service,
    types::{request, response},
};
use crate::types::Context;
use axum::{
    body::Body,
    extract::{Json, State},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use std::sync::Arc;

pub async fn handler(
    state: State<Arc<Context>>,
    TypedHeader(headers): TypedHeader<request::Headers>,
    body: Body,
) -> impl IntoResponse {
    let body = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| response::Error::ServerError)?;

    let Json(json) = Json::from_bytes(body.as_ref()).map_err(|_| response::Error::ServerError)?;

    let State(ctx) = state;

    service(
        ctx,
        request::Payload {
            body,
            json,
            headers,
        },
    )
    .await
}
