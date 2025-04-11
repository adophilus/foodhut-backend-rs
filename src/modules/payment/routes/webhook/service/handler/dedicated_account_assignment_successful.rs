use super::super::super::types::{response, DedicatedAccountAssignmentSuccessful};
use crate::{
    modules::{notification, user, wallet},
    types::Context,
};
use std::sync::Arc;

pub async fn handler(
    ctx: Arc<Context>,
    event: DedicatedAccountAssignmentSuccessful,
) -> response::Response {
    let user = user::repository::find_by_email(&ctx.db_conn.pool, event.customer.email.clone())
        .await
        .map_err(|_| response::Error::ServerError)?
        .ok_or_else(|| {
            tracing::error!(
                "User not found for dedicated account assignment: {}",
                &event.customer.email
            );
            response::Error::UserNotFound
        })?;

    wallet::repository::update_metatata_by_owner_id(
        &ctx.db_conn.pool,
        user.id.clone(),
        wallet::repository::WalletMetadata {
            backend: Some(wallet::repository::WalletBackend::Paystack(
                wallet::repository::PaystackWalletMetadata {
                    customer: wallet::repository::PaystackCustomer {
                        id: event.customer.id,
                        code: event.customer.code,
                    },
                    dedicated_account: wallet::repository::PaystackDedicatedAccount {
                        id: event.dedicated_account.id,
                        bank: wallet::repository::PaystackBank {
                            id: event.dedicated_account.bank.id,
                            name: event.dedicated_account.bank.name,
                            slug: event.dedicated_account.bank.slug,
                        },
                        account_name: event.dedicated_account.account_name,
                        account_number: event.dedicated_account.account_number,
                        active: event.dedicated_account.active,
                    },
                },
            )),
        },
    )
    .await
    .map_err(|_| response::Error::ServerError)?;

    tokio::spawn(notification::service::send(
        ctx.clone(),
        notification::service::Notification::bank_account_creation_successful(user),
        notification::service::Backend::Email,
    ));

    Ok(response::Success::Successful)
}
