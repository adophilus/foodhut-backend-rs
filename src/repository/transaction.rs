use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ulid::Ulid;

use crate::utils::database::DatabaseConnection;

#[derive(Serialize, Deserialize)]
struct OnlineTransaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub type_: TransactionType,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
struct WalletTransaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub type_: TransactionType,
    pub wallet_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Transaction {
    OnlineTransaction,
    WalletTransaction,
}

pub enum Error {
    UnexpectedError,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransactionType {
    #[serde(rename = "CREDIT")]
    Credit,
    #[serde(rename = "DEBIT")]
    Debit,
}

impl ToString for TransactionType {
    fn to_string(&self) -> String {
        match self {
            TransactionType::Credit => String::from("CREDIT"),
            TransactionType::Debit => String::from("DEBIT"),
        }
    }
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CREDIT" => Ok(TransactionType::Credit),
            "DEBIT" => Ok(TransactionType::Debit),
            _ => Err(format!("'{}' is not a valid TransactionType", s)),
        }
    }
}

impl From<String> for TransactionType {
    fn from(s: String) -> Self {
        s.parse()
            .unwrap_or_else(|_| panic!("Failed to parse '{}' into a TransactionType", s))
    }
}

#[derive(Debug)]
pub struct CreateOnlineTransactionPayload {
    pub amount: BigDecimal,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub user_id: String,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct CreateWalletTransactionPayload {
    pub amount: BigDecimal,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub wallet_id: String,
    pub note: Option<String>,
}

#[derive(Debug)]
pub enum CreatePayload {
    CreateWalletTransactionPayload,
    CreateOnlineTransactionPayload,
}

pub async fn create(db: DatabaseConnection, payload: CreatePayload) -> Result<Transaction, Error> {
    match payload {
        CreatePayload::CreateOnlineTransactionPayload(payload) => {
            create_online_transaction(db, payload).await
        }
        CreatePayload::CreateWalletTransactionPayload(payload) => {
            create_wallet_transaction(db, payload).await
        }
    }
}

async fn create_online_transaction(
    db: DatabaseConnection,
    payload: CreateOnlineTransactionPayload,
) -> Result<Transaction, Error> {
    match sqlx::query_as!(
        Transaction,
        "INSERT INTO transactions (id, amount, type, note, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        Ulid::new().to_string(),
        payload.amount,
        payload.type_,
        payload.note,
        payload.user_id
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(transaction) => Ok(transaction),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to create transaction {:?}: {}",
                payload,
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}

async fn create_wallet_transaction(
    db: DatabaseConnection,
    payload: CreateWalletTransactionPayload,
) -> Result<Transaction, Error> {
    match sqlx::query_as!(
        Transaction,
        "INSERT INTO transactions (id, amount, type, note, wallet_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        Ulid::new().to_string(),
        payload.amount,
        payload.type_,
        payload.note,
        payload.wallet_id
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(transaction) => Ok(transaction),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to create transaction {:?}: {}",
                payload,
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}
