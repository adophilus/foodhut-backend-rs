use super::{email, push};
use crate::{repository, types};
use std::sync::Arc;

pub enum Backend {
    Email,
    Push,
}

#[derive(Clone)]
pub enum NotificationRecipient {
    SingleRecipient(repository::user::User),
}

#[derive(Clone)]
pub enum NotificationType {
    Registered,
    OrderPaid { order: repository::order::Order },
    PasswordResetRequested { user: repository::user::User },
}

#[derive(Clone)]
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

    pub fn password_reset_requested(user: repository::user::User) -> Self {
        Self {
            type_: NotificationType::PasswordResetRequested { user: user.clone() },
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
