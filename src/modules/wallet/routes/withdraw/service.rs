use super::{
    super::super::repository,
    types::{request, response},
};
use crate::{
    modules::{kitchen, payment, transaction, user::repository::User},
    types::Context,
};
use bigdecimal::BigDecimal;
use std::sync::Arc;

struct WithdrawFundsPayload {
    account_number: String,
    bank_code: String,
    account_name: String,
    amount: BigDecimal,
    user: User,
    as_kitchen: bool,
}

pub async fn withdraw_funds(
    ctx: Arc<Context>,
    payload: WithdrawFundsPayload,
) -> response::Response {
    let mut tx = ctx
        .db_conn
        .pool
        .begin()
        .await
        .map_err(|_| response::Error::FailedToWithdrawFunds)?;

    let wallet = match payload.as_kitchen {
        true => {
            let kitchen = kitchen::repository::find_by_owner_id(&mut *tx, payload.user.id.clone())
                .await
                .map_err(|_| response::Error::FailedToWithdrawFunds)?
                .ok_or(response::Error::FailedToWithdrawFunds)?;
            repository::find_by_kitchen_id(&mut *tx, kitchen.id).await
        }
        false => repository::find_by_owner_id(&mut *tx, payload.user.id.clone()).await,
    }
    .map_err(|_| response::Error::FailedToWithdrawFunds)?
    .ok_or(response::Error::FailedToWithdrawFunds)?;

    if wallet.balance < payload.amount {
        return Err(response::Error::InsufficientFunds);
    }

    payment::service::online::withdraw_funds(
        ctx.clone(),
        payment::service::online::WithdrawFundsPayload {
            account_name: payload.account_name.clone(),
            account_number: payload.account_number.clone(),
            amount: payload.amount.clone(),
            bank_code: payload.bank_code.clone(),
            user: payload.user.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToWithdrawFunds)?;

    transaction::repository::create(
        &mut *tx,
        transaction::repository::CreatePayload::Wallet(
            transaction::repository::CreateWalletTransactionPayload {
                amount: payload.amount.clone(),
                direction: transaction::repository::TransactionDirection::Outgoing,
                note: Some(format!(
                    "Withdrawal to {} {}",
                    payload.account_name, payload.account_number
                )),
                purpose: Some(transaction::repository::TransactionPurpose::Other(
                    transaction::repository::TransactionPurposeOther,
                )),
                r#ref: None,
                wallet_id: wallet.id.clone(),
                user_id: payload.user.id.clone(),
            },
        ),
    )
    .await
    .map_err(|_| response::Error::FailedToWithdrawFunds)?;

    repository::update_by_id(
        &mut *tx,
        wallet.id.clone(),
        repository::UpdateByIdPayload {
            operation: repository::UpdateOperation::Debit,
            amount: payload.amount.clone(),
        },
    )
    .await
    .map_err(|_| response::Error::FailedToWithdrawFunds)?;

    tx.commit().await.map_err(|err| {
        tracing::error!("Failed to commit database transaction: {:?}", err);
        response::Error::FailedToWithdrawFunds
    })?;

    Ok(response::Success::WithdrawalPlaced)
}

pub async fn service(ctx: Arc<Context>, payload: request::Payload) -> response::Response {
    withdraw_funds(
        ctx.clone(),
        WithdrawFundsPayload {
            account_number: payload.body.account_number,
            bank_code: payload.body.bank_code,
            account_name: payload.body.account_name,
            amount: payload.body.amount,
            user: payload.auth.user,
            as_kitchen: payload.body.as_kitchen,
        },
    )
    .await
}
