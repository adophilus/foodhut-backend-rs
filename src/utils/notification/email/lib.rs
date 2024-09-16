use super::super::{Error, Notification, Result};
use crate::utils::notification;
use crate::{repository::password_reset, types};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Message,
};
use lettre::{AsyncTransport, Tokio1Executor};
use std::sync::Arc;

pub mod jobs {
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use std::sync::Arc;

    use crate::types;
    use hyper::StatusCode;

    #[derive(Clone)]
    struct RefreshToken {
        ctx: Arc<types::Context>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct RefreshTokenServerResponse {
        access_token: String,
    }

    impl types::SchedulableJob for RefreshToken {
        fn schedule(&self) -> apalis::cron::Schedule {
            apalis::cron::Schedule::from_str("0 0/30 * * * *").expect("Couldn't create schedule!")
        }

        async fn run(&self) -> Result<(), apalis::prelude::Error> {
            tracing::debug!("Attempting to refresh token...");
            let params = [
                ("client_id", self.ctx.google.client_id.clone()),
                ("client_secret", self.ctx.google.client_secret.clone()),
                ("refresh_token", self.ctx.mail.refresh_token.clone()),
                ("grant_type", "refresh_token".to_string()),
            ];

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
                                let formatted_err =
                                    format!("Failed to refresh mail access_token: {}", data);
                                tracing::error!(formatted_err);
                                return Err(apalis::prelude::Error::WorkerError(
                                    apalis::prelude::WorkerError::ProcessingError(formatted_err),
                                ));
                            }
                            Err(err) => {
                                let formatted_err = format!("Failed to get response body: {}", err);
                                tracing::error!(formatted_err);
                                return Err(apalis::prelude::Error::WorkerError(
                                    apalis::prelude::WorkerError::ProcessingError(formatted_err),
                                ));
                            }
                        }
                    } else {
                        match res.text().await {
                            Ok(data) => {
                                match serde_json::from_str::<RefreshTokenServerResponse>(&data) {
                                    Ok(structured_data) => {
                                        *self.ctx.mail.access_token.lock().unwrap() =
                                            structured_data.access_token;
                                        tracing::debug!("Successfully refreshed token");
                                        return Ok(());
                                    }
                                    Err(err) => {
                                        let formatted_err =
                                            format!("Failed to get response body: {}", err);
                                        tracing::error!(formatted_err);
                                        return Err(apalis::prelude::Error::WorkerError(
                                            apalis::prelude::WorkerError::ProcessingError(
                                                formatted_err,
                                            ),
                                        ));
                                    }
                                }
                            }
                            Err(err) => {
                                let formatted_err = format!("Failed to get response body: {}", err);
                                tracing::error!(formatted_err);
                                return Err(apalis::prelude::Error::WorkerError(
                                    apalis::prelude::WorkerError::ProcessingError(formatted_err),
                                ));
                            }
                        }
                    }
                }
                Err(err) => {
                    let formatted_err = format!(
                        "Error occurred while trying to send request to refresh token: {:?}",
                        err
                    );
                    tracing::error!(formatted_err);
                    return Err(apalis::prelude::Error::WorkerError(
                        apalis::prelude::WorkerError::ProcessingError(formatted_err),
                    ));
                }
            }
        }
    }

    pub async fn list(ctx: Arc<types::Context>) -> Vec<impl types::SchedulableJob> {
        vec![RefreshToken { ctx }]
    }
}

pub async fn send(ctx: Arc<types::Context>, notification: Notification) -> Result<()> {
    match notification.clone() {
        Notification::Registered(n) => send_registered_email(ctx, n).await,
        Notification::OrderPaid(n) => {
            unimplemented!()
        }
        notification::Notification::PasswordResetRequested(n) => unimplemented!(),
        notification::Notification::VerificationOtpRequested(n) => unimplemented!(),
    }
}

async fn send_registered_email(
    ctx: Arc<types::Context>,
    _notification: notification::types::Registered,
) -> Result<()> {
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
            _notification.user.first_name.clone(),
            _notification.user.last_name.clone(),
            _notification.user.email.clone()
        )
        .parse()
        .unwrap())
        .subject("Welcome to FoodHut")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "Greetings {}, welcome to FoodHut",
            _notification.user.first_name
        ))
        .unwrap();

    let access_token = {
        let token = ctx.mail.access_token.lock().unwrap().clone();
        token
    };
    let transport: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .authentication(vec![Mechanism::Xoauth2])
            .credentials(Credentials::new(
                ctx.mail.sender_email.clone(),
                access_token,
            ))
            .build();

    match transport.send(email).await {
        Ok(res) => Ok(()),
        Err(err) => {
            tracing::error!("Failed to send email: {}", err);
            Err(Error::NotSent)
        }
    }
}

async fn send_password_reset_requested_email(
    ctx: Arc<types::Context>,
    _notification: notification::types::PasswordResetRequested,
) -> Result<()> {
    let password_reset_link = format!(
        "{}/api/reset-password/{}",
        ctx.app.url, _notification.password_reset.code
    );
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
            _notification.user.first_name.clone(),
            _notification.user.last_name.clone(),
            _notification.user.email.clone()
        )
        .parse()
        .unwrap())
        .subject("Password Reset Requested")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "Use this link to reset your password: <a href=\"{}\">{}</a>",
            password_reset_link, password_reset_link
        ))
        .unwrap();

    let access_token = {
        let token = ctx.mail.access_token.lock().unwrap().clone();
        token
    };
    let transport: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .authentication(vec![Mechanism::Xoauth2])
            .credentials(Credentials::new(
                ctx.mail.sender_email.clone(),
                access_token,
            ))
            .build();

    match transport.send(email).await {
        Ok(res) => Ok(()),
        Err(err) => {
            tracing::error!("Failed to send email: {}", err);
            Err(Error::NotSent)
        }
    }
}
