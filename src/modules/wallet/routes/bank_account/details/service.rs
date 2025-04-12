use super::types::{request, response};
use crate::{modules::payment, types::Context};
use axum::http::{HeaderMap, Method, StatusCode};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct PaystackBankAccountDetailsResponse {
    status: bool,
    data: response::BankAccountDetails,
}

async fn get_bank_account_details(ctx: Arc<Context>, payload: request::Body) -> response::Response {
    let mut headers = HeaderMap::new();
    let auth_header = format!("Bearer {}", ctx.payment.secret_key);
    headers.insert(
        "Authorization",
        auth_header
            .clone()
            .try_into()
            .map_err(|_| response::Error::FailedToFetchBankAccountDetails)?,
    );
    headers.insert(
        "Content-Type",
        "application/json"
            .try_into()
            .map_err(|_| response::Error::FailedToFetchBankAccountDetails)?,
    );

    payment::utils::send_paystack_request::<PaystackBankAccountDetailsResponse>(
        ctx.clone(),
        payment::utils::SendPaystackRequestPayload {
            body: None,
            method: Method::GET,
            route: String::from("/bank/resolve"),
            query: Some(&[
                ("account_number", &payload.account_number),
                ("bank_code", &payload.bank_code),
            ]),
            expected_status_code: StatusCode::OK,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToFetchBankAccountDetails)
    .map(|res| {
        if !res.status {
            tracing::error!(
                "Failed to fetch paystack bank account details, false status: {:?}",
                res.data
            );
            return Err(response::Error::FailedToFetchBankAccountDetails);
        }
        Ok(response::Success::AccountDetails(res.data))
    })?
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    get_bank_account_details(ctx, payload.body).await
}
