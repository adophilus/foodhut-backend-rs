use super::types::response;
use crate::{modules::zoho, types::Context};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>) -> response::Response {
    zoho::service::generate_token_link(ctx)
        .await
        .map_err(|_| response::Error::FailedToGenerateToken)
        .map(response::Success::TokenGenerated)
}
