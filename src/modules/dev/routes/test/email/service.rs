use super::types::response;
use crate::{
    modules::{auth::middleware::Auth, notification},
    types::Context,
};
use std::sync::Arc;

pub async fn service(ctx: Arc<Context>, auth: Auth) -> response::Response {
    notification::service::send(
        ctx,
        notification::service::Notification::registered(auth.user),
        notification::service::Backend::Email,
    )
    .await
    .map(|_| response::Success::EmailSent)
    .map_err(|_| response::Error::FailedToSendEmail)
}
