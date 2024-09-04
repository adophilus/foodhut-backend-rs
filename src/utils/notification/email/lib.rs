use super::super::{Error, Notification, Result};
use crate::types;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Message,
};
use lettre::{AsyncTransport, Tokio1Executor};
use std::sync::Arc;

pub mod jobs {
    use std::str::FromStr;
    use std::sync::Arc;

    use crate::types;
    use apalis::cron::Schedule;
    use hyper::StatusCode;

    #[derive(Clone)]
    struct RefreshToken {
        ctx: Arc<types::Context>,
    }

    #[async_trait::async_trait]
    impl types::SchedulableJob for RefreshToken {
        fn schedule() -> apalis::cron::Schedule {
            Schedule::from_str("* * * * * *").expect("Couldn't start the scheduler!")
        }

        async fn run() {
            tracing::info!(
                "Attempting to refresh token... {}",
                self.ctx.mail.refresh_endpoint.clone()
            );
            let params = 
                    [("client_id", self.ctx.google.client_id),(client_secret,self.ctx.google.client_sec),(refresh_token,self.ctx.mail.refresh_token),("grant_type","refresh_token")],

            match reqwest::Client::new()
                .post(self.ctx.mail.refresh_endpoint.clone())
                .form(&params)
                .send()
                .await
            {
                Ok(res) => {
                    if res.status() != StatusCode::OK {
                        match res.text().await {
                            Ok(data) => {
                                tracing::error!("Failed to refresh mail access_token: {}", data);
                            }
                            Err(err) => {
                                tracing::error!("Failed to get response body: {}", err);
                            }
                        }
                    } else {
                        match res.text().await {
                            Ok(data) => {
                                tracing::info!("Server response: {}", data);
                            }
                            Err(err) => {
                                tracing::error!("Failed to get response body: {}", err);
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(
                        "Error occurred while trying to send request to refresh token: {:?}",
                        err
                    );
                }
            }
        }
    }

    pub async fn list(ctx: Arc<types::Context>) -> Vec<impl types::SchedulableJob> {
        vec![RefreshToken { ctx }]
    }
}

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
