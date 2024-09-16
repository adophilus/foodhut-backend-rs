use super::{email, push, sms};
use crate::{repository, types::Context};
use std::sync::Arc;

pub enum Backend {
    Email,
    Push,
    Sms,
}

pub mod types {
    use super::repository;

    #[derive(Clone)]
    pub struct Registered {
        pub user: repository::user::User,
    }

    #[derive(Clone)]
    pub struct OrderPaid {
        pub order: repository::order::Order,
    }

    #[derive(Clone)]
    pub struct PasswordResetRequested {
        pub user: repository::user::User,
        pub password_reset: repository::password_reset::PasswordReset,
    }

    #[derive(Clone)]
    pub struct VerificationOtpRequested {
        pub user: repository::user::User,
    }
}

#[derive(Clone)]
pub enum Notification {
    Registered(types::Registered),
    OrderPaid(types::OrderPaid),
    PasswordResetRequested(types::PasswordResetRequested),
    VerificationOtpRequested(types::VerificationOtpRequested),
}

impl Notification {
    pub fn registered(user: repository::user::User) -> Self {
        Notification::Registered(types::Registered { user })
    }

    pub fn password_reset_requested(
        user: repository::user::User,
        password_reset: repository::password_reset::PasswordReset,
    ) -> Self {
        Notification::PasswordResetRequested(types::PasswordResetRequested {
            user,
            password_reset,
        })
    }

    pub fn verification_otp_requested(user: repository::user::User) -> Self {
        Notification::VerificationOtpRequested(types::VerificationOtpRequested { user })
    }
}

pub enum Error {
    NotSent,
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO: create a general `NotificationResponse` type and make all notification backends return that

pub async fn send(ctx: Arc<Context>, notification: Notification, backend: Backend) -> Result<()> {
    match backend {
        Backend::Email => email::send(ctx, notification).await,
        Backend::Push => push::send(ctx, notification).await,
        Backend::Sms => {
            sms::send(ctx, notification).await;
            Ok(())
        }
    }
}
