pub mod email;
pub mod sms;
pub mod push;

// use super::{email, push, sms};
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
    pub struct VerificationOtpRequested {
        pub user: repository::user::User,
    }

    #[derive(Clone)]
    pub struct CustomerIdentificationFailed {
        pub user: repository::user::User,
        pub reason: String,
    }

    #[derive(Clone)]
    pub struct BankAccountCreationSuccessful {
        pub user: repository::user::User,
    }

    #[derive(Clone)]
    pub struct BankAccountCreationFailed {
        pub user: repository::user::User,
    }
}

#[derive(Clone)]
pub enum Notification {
    Registered(types::Registered),
    OrderPaid(types::OrderPaid),
    VerificationOtpRequested(types::VerificationOtpRequested),
    CustomerIdentificationFailed(types::CustomerIdentificationFailed),
    BankAccountCreationSuccessful(types::BankAccountCreationSuccessful),
    BankAccountCreationFailed(types::BankAccountCreationFailed),
}

impl Notification {
    pub fn registered(user: repository::user::User) -> Self {
        Notification::Registered(types::Registered { user })
    }

    pub fn verification_otp_requested(user: repository::user::User) -> Self {
        Notification::VerificationOtpRequested(types::VerificationOtpRequested { user })
    }

    pub fn customer_identification_failed(user: repository::user::User, reason: String) -> Self {
        Notification::CustomerIdentificationFailed(types::CustomerIdentificationFailed {
            user,
            reason,
        })
    }

    pub fn bank_account_creation_successful(user: repository::user::User) -> Self {
        Notification::BankAccountCreationSuccessful(types::BankAccountCreationSuccessful { user })
    }

    pub fn bank_account_creation_failed(user: repository::user::User) -> Self {
        Notification::BankAccountCreationFailed(types::BankAccountCreationFailed { user })
    }
}

pub enum Error {
    NotSent,
    InvalidNotification,
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
