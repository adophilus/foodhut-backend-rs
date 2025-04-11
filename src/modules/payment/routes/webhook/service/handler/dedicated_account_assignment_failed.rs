use super::super::super::types::{response, DedicatedAccountAssignmentFailed};
use crate::{
    modules::{notification, user},
    types::Context,
};
use std::sync::Arc;

pub async fn handler(
    ctx: Arc<Context>,
    event: DedicatedAccountAssignmentFailed,
) -> response::Response {
    let user = user::repository::find_by_email(&ctx.db_conn.pool, event.customer.email.clone())
        .await
        .map_err(|_| response::Error::ServerError)?
        .ok_or_else(|| {
            tracing::error!(
                "User not found for dedicated account assignment: {}",
                &event.customer.email
            );
            response::Error::UserNotFound
        })?;

    notification::service::send(
        ctx.clone(),
        notification::service::Notification::bank_account_creation_failed(user),
        notification::service::Backend::Email,
    )
    .await;

    Ok(response::Success::Successful)
}
