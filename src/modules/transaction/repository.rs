use crate::{
    define_paginated,
    utils::pagination::{Paginated, Pagination},
};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use std::convert::Into;
use std::str::FromStr;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all(serialize = "UPPERCASE", deserialize = "UPPERCASE"))]
pub enum TransactionType {
    Online,
    Wallet,
}

impl From<String> for TransactionType {
    fn from(s: String) -> Self {
        s.parse().unwrap()
    }
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ONLINE" => Ok(TransactionType::Online),
            "WALLET" => Ok(TransactionType::Wallet),
            _ => Err(format!("'{}' is not a valid TransactionType", s)),
        }
    }
}

impl ToString for TransactionType {
    fn to_string(&self) -> String {
        match self {
            TransactionType::Online => "ONLINE".to_string(),
            TransactionType::Wallet => "WALLET".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all(serialize = "UPPERCASE", deserialize = "UPPERCASE"))]
pub enum TransactionDirection {
    Outgoing,
    Incoming,
}

impl From<String> for TransactionDirection {
    fn from(s: String) -> Self {
        s.parse().unwrap()
    }
}

impl FromStr for TransactionDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OUTGOING" => Ok(TransactionDirection::Outgoing),
            "INCOMING" => Ok(TransactionDirection::Incoming),
            _ => Err(format!("'{}' is not a valid TransactionType", s)),
        }
    }
}

impl ToString for TransactionDirection {
    fn to_string(&self) -> String {
        match self {
            TransactionDirection::Incoming => "INCOMING".to_string(),
            TransactionDirection::Outgoing => "OUTGOING".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnlineTransaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub direction: TransactionDirection,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletTransaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub direction: TransactionDirection,
    pub wallet_id: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DbTransaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub direction: TransactionDirection,
    pub r#type: TransactionType,
    pub wallet_id: Option<String>,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

define_paginated!(DatabasePaginatedDbTransaction, DbTransaction);

impl From<DbTransaction> for Transaction {
    fn from(db_tx: DbTransaction) -> Self {
        serde_json::de::from_str(&serde_json::json!(db_tx).to_string())
            .map_err(|e| format!("Invalid transaction type found for {:?}: {}", db_tx, e))
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Transaction {
    Online(OnlineTransaction),
    Wallet(WalletTransaction),
}

pub enum Error {
    UnexpectedError,
}

#[derive(Debug)]
pub struct CreateOnlineTransactionPayload {
    pub amount: BigDecimal,
    pub direction: TransactionDirection,
    pub user_id: String,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct CreateWalletTransactionPayload {
    pub amount: BigDecimal,
    pub direction: TransactionDirection,
    pub wallet_id: String,
    pub note: Option<String>,
}

#[derive(Debug)]
pub enum CreatePayload {
    Online(CreateOnlineTransactionPayload),
    Wallet(CreateWalletTransactionPayload),
}

pub async fn create<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreatePayload,
) -> Result<Transaction, Error> {
    match payload {
        CreatePayload::Online(payload) => create_online_transaction(e, payload).await,
        CreatePayload::Wallet(payload) => create_wallet_transaction(e, payload).await,
    }
}

async fn create_online_transaction<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreateOnlineTransactionPayload,
) -> Result<Transaction, Error> {
    sqlx::query_as!(
        DbTransaction,
        "
        INSERT INTO transactions (id, amount, direction, type, note, user_id)
        VALUES ($1, $2, $3, $4, $5, $6) RETURNING *
        ",
        Ulid::new().to_string(),
        payload.amount,
        payload.direction.to_string(),
        TransactionType::Online.to_string(),
        payload.note,
        payload.user_id
    )
    .fetch_one(e)
    .await
    .map(Transaction::from)
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to create transaction {:?}: {}",
            payload,
            err
        );
        Error::UnexpectedError
    })
}

async fn create_wallet_transaction<'e, E: PgExecutor<'e>>(
    e: E,
    payload: CreateWalletTransactionPayload,
) -> Result<Transaction, Error> {
    sqlx::query_as!(
        DbTransaction,
        "INSERT INTO transactions (id, amount, type, note, wallet_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        Ulid::new().to_string(),
        payload.amount,
        payload.direction.to_string(),
        payload.note,
        payload.wallet_id
    )
    .fetch_one(e)
    .await
    .map(Into::into)
    .map_err(|err|{
        tracing::error!(
            "Error occurred while trying to create transaction {:?}: {}",
            payload,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_by_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
) -> Result<Option<Transaction>, Error> {
    sqlx::query_as!(
        DbTransaction,
        "SELECT * FROM transactions WHERE id = $1",
        id
    )
    .fetch_optional(e)
    .await
    .map(|db_transaction| db_transaction.map(Into::into))
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch transaction by id {}: {:?}",
            id,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_by_id_and_user_id<'e, E: PgExecutor<'e>>(
    e: E,
    id: String,
    user_id: String,
) -> Result<Option<Transaction>, Error> {
    sqlx::query_as!(
        DbTransaction,
        "SELECT * FROM transactions WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .fetch_optional(e)
    .await
    .map(|db_transaction| db_transaction.map(Into::into))
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch transaction by id {}: {:?}",
            id,
            err
        );
        Error::UnexpectedError
    })
}

#[derive(Deserialize)]
pub struct FindManyFilters {
    pub user_id: Option<String>,
    pub before: Option<u64>,
    pub after: Option<u64>,
}

pub async fn find_many<'e, Executor: PgExecutor<'e>>(
    e: Executor,
    pagination: Pagination,
    filters: FindManyFilters,
) -> Result<Paginated<Transaction>, Error> {
    sqlx::query_as!(
        DatabasePaginatedDbTransaction,
        r#"
        WITH filtered_transactions AS (
            SELECT transactions.*
            FROM transactions
            WHERE
                ($3::TEXT IS NULL OR transactions.user_id = $3)
                AND ($4::BIGINT IS NULL OR EXTRACT(EPOCH FROM transactions.created_at) < $4)
                AND ($5::BIGINT IS NULL OR EXTRACT(EPOCH FROM transactions.created_at) > $5)
            ORDER BY created_at DESC
            LIMIT $2
            OFFSET ($1 - 1) * $2
        ),
        total_count AS (
            SELECT COUNT(transactions.id) AS total_rows
            FROM transactions
            WHERE
                ($3::TEXT IS NULL OR transactions.user_id = $3)
                AND ($4::BIGINT IS NULL OR EXTRACT(EPOCH FROM transactions.created_at) < $4)
                AND ($5::BIGINT IS NULL OR EXTRACT(EPOCH FROM transactions.created_at) > $5)
        )
        SELECT 
            COALESCE(JSONB_AGG(ROW_TO_JSON(filtered_transactions)), '[]'::jsonb) AS items,
            JSONB_BUILD_OBJECT(
                'page', $1,
                'per_page', $2,
                'total', (SELECT total_rows FROM total_count)
            ) AS meta
        FROM filtered_transactions;
    "#,
        pagination.page as i32,
        pagination.per_page as i32,
        filters.user_id,
        filters.before.map(|before| before as i64),
        filters.after.map(|after| after as i64),
    )
    .fetch_one(e)
    .await
    .map(|paginated_db_transaction| {
        Paginated::new(
            paginated_db_transaction
                .items
                .0
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
            paginated_db_transaction.meta.total,
            paginated_db_transaction.meta.page,
            paginated_db_transaction.meta.per_page,
        )
    })
    .map_err(|err| {
        tracing::error!("Error occurred while trying to fetch transactions: {}", err);
        Error::UnexpectedError
    })
}
