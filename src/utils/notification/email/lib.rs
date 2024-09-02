use super::super::{Error, Notification, Result};
use crate::types;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Message,
};
use lettre::{AsyncTransport, Tokio1Executor};
use std::sync::Arc;

pub async fn refresh_token() {}

pub async fn send(ctx: Arc<types::Context>, notification: Notification) -> Result<()> {
    match notification.recipient {
        crate::utils::notification::NotificationRecipient::SingleRecipient(recipient) => {
            let email = Message::builder()
                .from(
                    format!(
                        "{} <{}>",
                        ctx.mail.sender_name.clone(),
                        ctx.mail.sender_email.clone()
                    )
                    .parse()
                    .unwrap(),
                )
                .to(format!(
                    "{} {} <{}>",
                    recipient.first_name.clone(),
                    recipient.last_name.clone(),
                    recipient.email.clone()
                )
                .parse()
                .unwrap())
                .subject("Welcome to FoodHut")
                .header(ContentType::TEXT_HTML)
                .body(String::from("Welcome to FoodHut"))
                .unwrap();

            let transport: AsyncSmtpTransport<Tokio1Executor> =
                AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
                    .unwrap()
                    .authentication(vec![Mechanism::Xoauth2])
                    .credentials(Credentials::new(
                        ctx.mail.sender_email.clone(),
                        ctx.mail.access_token.clone(),
                    ))
                    .build();

            match transport.send(email).await {
                Ok(res) => Ok(()),
                Err(err) => {
                    tracing::error!("Failed to send email: {:?}", err);
                    Err(Error::NotSent)
                }
            }
        }
    }
}
