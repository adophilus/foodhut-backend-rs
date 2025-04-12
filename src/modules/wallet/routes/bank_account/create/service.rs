use super::types::{request, response};
use crate::{
    modules::{payment, user::repository::User, wallet},
    types::{AppEnvironment, Context},
};
use axum::http::{HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitializeBankAccountCreationServiceResponse {
    status: bool,
    message: String,
}

struct RequestVirtualAccountPayload {
    pub bvn: String,
    pub bank_code: String,
    pub account_number: String,
    pub user: User,
}

async fn request_virtual_account(
    ctx: Arc<Context>,
    payload: RequestVirtualAccountPayload,
) -> response::Response {
    let _wallet = wallet::repository::find_by_owner_id(&ctx.db_conn.pool, payload.user.id.clone())
        .await
        .map_err(|_| response::Error::FailedToSendApplication(None))?
        .ok_or(response::Error::FailedToSendApplication(None))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", ctx.payment.secret_key.clone())
            .try_into()
            .map_err(|_| response::Error::FailedToSendApplication(None))?,
    );

    let preferred_bank = match ctx.app.environment {
        AppEnvironment::Production => "titan-paystack",
        AppEnvironment::Development => "test-bank",
    };

    payment::utils::send_paystack_request::<InitializeBankAccountCreationServiceResponse>(
        ctx.clone(),
        payment::utils::SendPaystackRequestPayload {
            expected_status_code: StatusCode::OK,
            route: String::from("/dedicated_account/assign"),
            method: Method::POST,
            body: Some(
                json!({
                    "country": "NG",
                    "type": "bank_account",
                    "account_number": payload.account_number,
                    "bvn": payload.bvn,
                    "bank_code": payload.bank_code,
                    "first_name": payload.user.first_name,
                    "last_name": payload.user.last_name,
                    "email": payload.user.email,
                    "phone": payload.user.phone_number,
                    "preferred_bank": preferred_bank,
                })
                .to_string(),
            ),
            query: None,
        },
    )
    .await
    .map_err(|_| response::Error::FailedToSendApplication(None))
    .map(|res| {
        if !res.status {
            return Err(response::Error::FailedToSendApplication(Some(res.message)));
        }

        Ok(response::Success::ApplicationSent)
    })?
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    request_virtual_account(
        ctx,
        RequestVirtualAccountPayload {
            bvn: payload.body.bvn,
            bank_code: payload.body.bank_code,
            account_number: payload.body.account_number,
            user: payload.auth.user,
        },
    )
    .await
}
