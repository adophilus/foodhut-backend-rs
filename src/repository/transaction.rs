use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::utils::database::DatabaseConnection;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    pub amount: BigDecimal,
    pub note: Option<String>,
    pub wallet_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub enum Error {
    UnexpectedError,
}

#[derive(Debug)]
pub struct CreatePayload {
    pub amount: BigDecimal,
    pub wallet_id: String,
    pub note: Option<String>,
}

pub async fn create(db: DatabaseConnection, payload: CreatePayload) -> Result<Transaction, Error> {
    match sqlx::query_as!(
        Transaction,
        "INSERT INTO transactions (id, amount, note, wallet_id) VALUES ($1, $2, $3, $4) RETURNING *",
        Ulid::new().to_string(),
        payload.amount,
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
