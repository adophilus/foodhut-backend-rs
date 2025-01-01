use crate::{modules::user::repository::User, Context};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Message,
};
use lettre::{AsyncTransport, Tokio1Executor};
use std::sync::Arc;

use super::{types, Error, Notification, Result};

pub mod job {
    use crate::types::{Context, SchedulableJob};
    use std::sync::Arc;

    pub fn list(_: Arc<Context>) -> Vec<SchedulableJob> {
        vec![]
    }
}

// TODO: handle other notification types
pub async fn send(ctx: Arc<Context>, notification: Notification) -> Result<()> {
    match notification.clone() {
        Notification::Registered(n) => send_registered_email(ctx, n).await,
        // Notification::OrderPaid(_) => unimplemented!(),
        Notification::VerificationOtpRequested(_) => Err(Error::InvalidNotification),
        // Notification::CustomerIdentificationFailed(n) => {
        //     send_customer_identification_failed_email(ctx, n).await
        // }
        Notification::BankAccountCreationFailed(n) => {
            send_bank_account_creation_failed_email(ctx, n).await
        }
        Notification::BankAccountCreationSuccessful(n) => {
            send_bank_account_creation_successful_email(ctx, n).await
        }
    }
}

struct SendEmailPayload {
    user: User,
    body: String,
    subject: String,
}

async fn send_email(ctx: Arc<Context>, payload: SendEmailPayload) -> Result<()> {
    tracing::warn!("{}", ctx.mail.user.clone());
    tracing::warn!("{}", ctx.mail.password.clone());

    let email = Message::builder()
        .from(
            format!("{} <{}>", ctx.mail.sender.clone(), ctx.mail.user.clone())
                .parse()
                .unwrap(),
        )
        .to(format!(
            "{} {} <{}>",
            payload.user.first_name.clone(),
            payload.user.last_name.clone(),
            payload.user.email.clone()
        )
        .parse()
        .unwrap())
        .subject(payload.subject)
        .header(ContentType::TEXT_HTML)
        .body(payload.body)
        .unwrap();

    let transport: AsyncSmtpTransport<Tokio1Executor> =
        // AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        AsyncSmtpTransport::<Tokio1Executor>::relay("foodhut.co")
            .unwrap()
            .authentication(vec![Mechanism::Plain])
            .credentials(Credentials::new(
                ctx.mail.user.clone(),
                ctx.mail.password.clone()
            ))
            .build();

    transport.send(email).await.map(|_| ()).map_err(|err| {
        tracing::error!("Failed to send email: {}", err);
        Error::NotSent
    })
}

async fn send_registered_email(ctx: Arc<Context>, _notification: types::Registered) -> Result<()> {
    send_email(
        ctx,
        SendEmailPayload {
            user: _notification.user.clone(),
            subject: String::from("Welcome to FoodHut"),
            body: format!(
                "Greetings {}, welcome to FoodHut",
                _notification.user.first_name
            ),
        },
    )
    .await
}

async fn send_customer_identification_failed_email(
    ctx: Arc<Context>,
    _notification: types::CustomerIdentificationFailed,
) -> Result<()> {
    send_email(
        ctx,
        SendEmailPayload {
            user: _notification.user.clone(),
            subject: String::from("Virtual Account Creation Request Failed"),
            body: format!(
                "Dear customer, you request to create a virtual account failed because: {}",
                _notification.reason,
            ),
        },
    )
    .await
}

async fn send_bank_account_creation_failed_email(
    ctx: Arc<Context>,
    _notification: types::BankAccountCreationFailed,
) -> Result<()> {
    send_email(
        ctx,
        SendEmailPayload {
            user: _notification.user.clone(),
            subject: String::from("Virtual Account Creation Failed"),
            body: String::from("Dear customer, your virtual account couldn't be created"),
        },
    )
    .await
}

async fn send_bank_account_creation_successful_email(
    ctx: Arc<Context>,
    _notification: types::BankAccountCreationSuccessful,
) -> Result<()> {
    send_email(
        ctx,
        SendEmailPayload {
            user: _notification.user.clone(),
            subject: String::from("Virtual Account Created"),
            body: String::from("Dear customer, your virtual account has been created!"),
        },
    )
    .await
}
