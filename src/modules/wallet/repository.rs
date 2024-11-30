// TODO: switch to the new database paginated macro

use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{types::BigDecimal, PgExecutor};
use std::convert::Into;
use ulid::Ulid;

use crate::utils::pagination::{Paginated, Pagination};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaystackBank {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaystackDedicatedAccount {
    pub id: i32,
    pub bank: PaystackBank,
    pub account_name: String,
    pub account_number: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaystackCustomer {
    pub id: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaystackWalletMetadata {
    pub customer: PaystackCustomer,
    pub dedicated_account: PaystackDedicatedAccount,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WalletBackend {
    #[serde(rename = "paystack")]
    Paystack(PaystackWalletMetadata),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WalletMetadata {
    pub backend: Option<WalletBackend>,
}

impl From<serde_json::Value> for WalletMetadata {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value::<Self>(value).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallet {
    pub id: String,
    pub balance: BigDecimal,
    pub metadata: WalletMetadata,
    pub owner_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create<'e, E: PgExecutor<'e>>(e: E, owner_id: String) -> Result<Wallet, Error> {
    sqlx::query_as!(
        Wallet,
        "
        INSERT INTO wallets (id, balance, metadata, owner_id)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        ",
        Ulid::new().to_string(),
        BigDecimal::from_u8(0).unwrap(),
        json!(WalletMetadata { backend: None }),
        owner_id,
    )
    .fetch_one(e)
    .await
    .map_err(|err| {
        tracing::error!("Error occurred while trying to create a wallet: {}", err);
        Error::UnexpectedError
    })
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(e: E, id: String) -> Result<Option<Wallet>, Error> {
    match sqlx::query_as!(Wallet, "SELECT * FROM wallets WHERE id = $1", id)
        .fetch_optional(e)
        .await
    {
        Ok(maybe_wallet) => Ok(maybe_wallet),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch a wallet by id {}: {}",
                id,
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_owner_id<'e, Executor: PgExecutor<'e>>(
    e: Executor,
    owner_id: String,
) -> Result<Option<Wallet>, Error> {
    sqlx::query_as!(
        Wallet,
        "SELECT * FROM wallets WHERE owner_id = $1",
        owner_id
    )
    .fetch_optional(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch a wallet by owner_id {}: {}",
            owner_id,
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Serialize)]
pub enum UpdateWalletOperation {
    #[serde(rename = "CREDIT")]
    Credit,
    #[serde(rename = "DEBIT")]
    Debit,
}

impl ToString for UpdateWalletOperation {
    fn to_string(&self) -> String {
        match *self {
            UpdateWalletOperation::Credit => String::from("CREDIT"),
            UpdateWalletOperation::Debit => String::from("DEBIT"),
        }
    }
}

#[derive(Serialize)]
pub struct UpdateWalletPayload {
    pub operation: UpdateWalletOperation,
    pub amount: BigDecimal,
}

pub async fn update_by_id<'e, Executor: PgExecutor<'e>>(
    e: Executor,
    id: String,
    payload: UpdateWalletPayload,
) -> Result<(), Error> {
    // TODO: checks need to be made so that the user's balance cannot be negative

    sqlx::query!(
        "
        UPDATE wallets SET
                balance = CASE WHEN $1 = $2 THEN balance + $3::numeric ELSE balance - $3::numeric END
        WHERE
            id = $4
        ",
        payload.operation.to_string(),
        UpdateWalletOperation::Credit.to_string(),
        payload.amount,
        id
    )
    .execute(e)
        .await
        .map(|_|())
    .map_err(|err|{
        tracing::error!("Error occurred while trying to update wallet by id: {}", err);
        Error::UnexpectedError
    })
}

pub async fn update_metatata_by_owner_id<'e, E: PgExecutor<'e>>(
    e: E,
    owner_id: String,
    payload: WalletMetadata,
) -> Result<Wallet, Error> {
    sqlx::query_as!(
        Wallet,
        r#"
        UPDATE wallets
        SET
            metadata = $1
        WHERE
            owner_id = $2
        RETURNING *
        "#,
        serde_json::to_value(payload).unwrap(),
        owner_id
    )
    .fetch_one(e)
    .await
    .map_err(|e| {
        tracing::error!(
            "Error occurred while trying to update wallet metadata: {}",
            e
        );
        Error::UnexpectedError
    })
}
