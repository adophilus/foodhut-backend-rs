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
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::pin::Pin;
    use std::str::FromStr;
    use std::sync::Arc;

    #[derive(Serialize, Deserialize, Debug)]
    struct RefreshTokenServerResponse {
        access_token: String,
    }

    async fn refresh_token_job(ctx: Arc<Context>) -> Result<(), apalis::prelude::Error> {
        tracing::debug!("Attempting to refresh token...");
        let params = [
            ("client_id", ctx.google.client_id.clone()),
            ("client_secret", ctx.google.client_secret.clone()),
            ("refresh_token", ctx.mail.refresh_token.clone()),
            ("grant_type", "refresh_token".to_string()),
        ];

        match reqwest::Client::new()
            .post(ctx.mail.refresh_endpoint.clone())
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
                                    *ctx.mail.access_token.lock().unwrap() =
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

    fn setup_refresh_token_job(
        ctx: Arc<Context>,
    ) -> Arc<
        dyn Fn() -> Pin<
                Box<dyn std::future::Future<Output = Result<(), apalis::prelude::Error>> + Send>,
            > + Send
            + Sync,
    > {
        Arc::new(move || {
            let ctx = ctx.clone();
            Box::pin(async move { refresh_token_job(ctx).await })
        })
    }

    pub fn list(ctx: Arc<Context>) -> Vec<SchedulableJob> {
        vec![SchedulableJob {
            schedule: apalis::cron::Schedule::from_str("0 0/30 * * * *")
                .expect("Couldn't create schedule!"),
            job: setup_refresh_token_job(ctx),
        }]
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
