use crate::utils::database::DatabaseConnection;
use chrono::{NaiveDateTime, Utc};
use ulid::Ulid;

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: NaiveDateTime,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, user_id: String) -> Result<Session, Error> {
    match sqlx::query_as!(
        Session,
        "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, $3) RETURNING *",
        Ulid::new().to_string(),
        user_id,
        Utc::now().naive_utc() + chrono::Duration::days(7)
    )
    .fetch_one(&db.pool)
    .await
    {
        Ok(session) => Ok(session),
        Err(e) => {
            log::error!(
                "Error occurred while creating a new session for user with id {}: {}",
                user_id,
                e
            );
            Err(Error::UnexpectedError)
        }
    }
}

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Option<Session> {
    sqlx::query_as!(Session, "SELECT * FROM sessions WHERE id = $1", id,)
        .fetch_optional(&db.pool)
        .await
        .map_err(|err| {
            log::error!(
                "Error occurred while fetching session with id {}: {}",
                id,
                err
            );
        })
        .unwrap_or(None)
}
