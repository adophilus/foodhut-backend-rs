use super::{
    service::service,
    types::{request, response},
};
use crate::types::Context;
use axum::extract::State;
use std::sync::Arc;

pub async fn handler(_: request::Auth, State(ctx): State<Arc<Context>>) -> response::Response {
    service(ctx).await
}
