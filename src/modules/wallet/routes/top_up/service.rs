use super::types::{request, response};
use crate::{
    modules::{payment, user::repository::User},
    types::Context,
};
use bigdecimal::BigDecimal;
use std::sync::Arc;

pub struct CreateTopupInvoicePayload {
    pub user: User,
    pub amount: BigDecimal,
}

pub async fn create_topup_invoice(
    ctx: Arc<Context>,
    payload: CreateTopupInvoicePayload,
) -> response::Response {
    payment::service::online::create_topup_invoice(
        ctx,
        payment::service::online::CreateTopupInvoicePayload {
            user: payload.user.clone(),
            amount: payload.amount.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToCreateTopupInvoiceLink)
    .map(response::Success::TopupInvoiceLink)
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    create_topup_invoice(
        ctx.clone(),
        CreateTopupInvoicePayload {
            amount: payload.body.amount,
            user: payload.auth.user,
        },
    )
    .await
}
