use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::BigDecimal;
use std::convert::Into;
use ulid::Ulid;

use crate::utils::{
    database::DatabaseConnection,
    pagination::{Paginated, Pagination},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaystackWalletDetails {
    pub customer_id: String,
    pub customer_code: String,
    pub account_number: String,
    pub bank_identifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WalletBackend {
    Paystack(PaystackWalletDetails),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WalletMetadata {
    pub backend: WalletBackend,
}

impl From<serde_json::Value> for WalletMetadata {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_str::<Self>(
            value
                .as_str()
                .expect("Not possible to decode from NULL data"),
        )
        .expect("Invalid WalletMetadata type")
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

pub struct CreateWalletPayload {
    pub owner_id: String,
    pub metadata: WalletMetadata,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, payload: CreateWalletPayload) -> Result<(), Error> {
    match sqlx::query!(
        "
        INSERT INTO wallets (
            id,
            balance,
            metadata,
            owner_id
        )
        VALUES ($1, $2, $3, $4)
    ",
        Ulid::new().to_string(),
        BigDecimal::from_u8(0).unwrap(),
        json!(payload.metadata),
        payload.owner_id,
    )
    .execute(&db.pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::error!("Error occurred while trying to create a wallet: {}", err);
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Wallet>, Error> {
    match sqlx::query_as!(Wallet, "SELECT * FROM wallets WHERE id = $1", id)
        .fetch_optional(&db.pool)
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

pub async fn find_by_owner_id(
    db: DatabaseConnection,
    owner_id: String,
) -> Result<Option<Wallet>, Error> {
    match sqlx::query_as!(
        Wallet,
        "SELECT * FROM wallets WHERE owner_id = $1",
        owner_id
    )
    .fetch_optional(&db.pool)
    .await
    {
        Ok(maybe_wallet) => Ok(maybe_wallet),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch a wallet by owner_id {}: {}",
                owner_id,
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCountedResult {
    data: Vec<Wallet>,
    total: u32,
}

impl Into<DatabaseCountedResult> for Option<serde_json::Value> {
    fn into(self) -> DatabaseCountedResult {
        match self {
            Some(json) => {
                serde_json::de::from_str::<DatabaseCountedResult>(json.to_string().as_ref())
                    .unwrap()
            }
            None => DatabaseCountedResult {
                data: vec![],
                total: 0,
            },
        }
    }
}

#[derive(Deserialize)]
struct DatabaseCounted {
    result: DatabaseCountedResult,
}

pub async fn find_many(
    db: DatabaseConnection,
    pagination: Pagination,
) -> Result<Paginated<Wallet>, Error> {
    match sqlx::query_as!(
        DatabaseCounted,
        "
            WITH filtered_data AS (
                SELECT *
                FROM wallets 
                LIMIT $1
                OFFSET $2
            ), 
            total_count AS (
                SELECT COUNT(id) AS total_rows
                FROM wallets
            )
            SELECT JSONB_BUILD_OBJECT(
                'data', COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_data)), '[]'::jsonb),
                'total', (SELECT total_rows FROM total_count)
            ) AS result
            FROM filtered_data;
        ",
        pagination.per_page as i64,
        ((pagination.page - 1) * pagination.per_page) as i64,
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(counted) => Ok(Paginated::new(
            counted.result.data,
            counted.result.total,
            pagination.page,
            pagination.per_page,
        )),
        Err(err) => {
            tracing::error!("Error occurred while trying to fetch many wallets: {}", err);
            Err(Error::UnexpectedError)
        }
    }
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

pub async fn update_by_id(
    db: DatabaseConnection,
    id: String,
    payload: UpdateWalletPayload,
) -> Result<(), Error> {
    // FIX: checks need to be made so that the user's balance cannot be negagtive

    match sqlx::query!(
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
    .execute(&db.pool)
    .await
    {
        Err(e) => {
            log::error!("Error occurred while trying to update wallet by id: {}", e);
            return Err(Error::UnexpectedError);
        }
        _ => Ok(()),
    }
}
