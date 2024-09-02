use super::{email, push};
use crate::{repository, types};
use std::sync::Arc;

pub enum Backend {
    Email,
    Push,
}

pub enum NotificationRecipient {
    SingleRecipient(repository::user::User),
}

pub enum NotificationType {
    Registered,
    OrderPaid { order: repository::order::Order },
}

pub struct Notification {
    pub type_: NotificationType,
    pub recipient: NotificationRecipient,
}

impl Notification {
    pub fn registered(user: repository::user::User) -> Self {
        Self {
            type_: NotificationType::Registered,
            recipient: NotificationRecipient::SingleRecipient(user),
        }
    }
}

pub enum Error {
    NotSent,
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn send(
    ctx: Arc<types::Context>,
    notification: Notification,
    backend: Backend,
) -> Result<()> {
    match backend {
        Backend::Email => email::send(ctx, notification).await,
        Backend::Push => push::send(ctx, notification).await,
    }
}
