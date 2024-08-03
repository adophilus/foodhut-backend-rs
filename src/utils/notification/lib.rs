use super::{email, push};
use crate::{repository, types};
use std::sync::Arc;

pub enum NotificationBackend {
    Email,
    Push,
}

pub enum NotificationRecipient {
    SingleRecipient(repository::user::User),
}

pub enum NotificationType {
    OrderPaid { order: repository::order::Order },
}

pub struct Notification {
    type_: NotificationType,
    recipient: NotificationRecipient,
}

pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn send(
    ctx: Arc<types::Context>,
    notification: Notification,
    backend: NotificationBackend,
) -> Result<()> {
    match backend {
        NotificationBackend::Email => email::send(ctx, notification).await,
        NotificationBackend::Push => push::send(ctx, notification).await,
    }
}
