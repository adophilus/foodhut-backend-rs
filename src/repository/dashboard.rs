use serde::{Deserialize, Serialize};

use crate::utils::database::DatabaseConnection;

pub enum Error {
    UnexpectedError,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseInfo {
    kitchens: i64,
    meals: i64,
    orders: i64,
    transactions: i64,
}

#[derive(Serialize, Deserialize)]
struct OptionalDatabaseInfo {
    kitchens: Option<i64>,
    meals: Option<i64>,
    orders: Option<i64>,
    transactions: Option<i64>,
}

pub async fn get_total_resources(db: DatabaseConnection) -> Result<DatabaseInfo, Error> {
    match sqlx::query_as!(
        OptionalDatabaseInfo,
        "
        SELECT
           	(SELECT COUNT(id) FROM kitchens) AS kitchens,
           	(SELECT COUNT(1) FROM meals) AS meals,
           	(SELECT COUNT(id) FROM orders) AS orders,
           	(SELECT COUNT(id) FROM transactions) AS transactions
    "
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(res) => Ok(DatabaseInfo {
            orders: res.orders.unwrap(),
            meals: res.meals.unwrap(),
            kitchens: res.kitchens.unwrap(),
            transactions: res.transactions.unwrap(),
        }),
        Err(err) => {
            tracing::error!(
                "Error occurred while trying to fetch info from tables: {}",
                err
            );
            Err(Error::UnexpectedError)
        }
    }
}
