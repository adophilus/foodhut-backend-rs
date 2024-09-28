use crate::utils::database::DatabaseConnection;
use chrono::{NaiveDateTime, Utc};
use ulid::Ulid;

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub refresh_token_expires_at: NaiveDateTime,
}

pub enum Error {
    UnexpectedError,
}

pub async fn create(db: DatabaseConnection, user_id: String) -> Result<Session, Error> {
    let id = Ulid::new().to_string();
    let access_token = id.clone();
    let refresh_token = Ulid::new().to_string();
    match sqlx::query_as!(
        Session,
        "INSERT INTO sessions (id, user_id, access_token, refresh_token, access_token_expires_at, refresh_token_expires_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        id,
        user_id,
        access_token,
        refresh_token,
        Utc::now().naive_utc() + chrono::Duration::days(15),
        Utc::now().naive_utc() + chrono::Duration::days(30)
    )
    .fetch_one(&db.pool)
    .await
    // TODO: fix this error handling here
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

pub async fn find_by_id(db: DatabaseConnection, id: String) -> Result<Option<Session>, Error> {
    sqlx::query_as!(Session, "SELECT * FROM sessions WHERE id = $1", id,)
        .fetch_optional(&db.pool)
        .await
        .map_err(|err| {
            log::error!(
                "Error occurred while fetching session with id {}: {}",
                id,
                err
            );
            Error::UnexpectedError
        })
}

pub async fn find_by_access_token(
    db: DatabaseConnection,
    access_token: String,
) -> Result<Option<Session>, Error> {
    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE access_token = $1",
        access_token
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|err| {
        log::error!(
            "Error occurred while fetching session with access_token {}: {}",
            access_token,
            err
        );
        Error::UnexpectedError
    })
}

pub async fn find_by_refresh_token(
    db: DatabaseConnection,
    refresh_token: String,
) -> Result<Option<Session>, Error> {
    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE refresh_token = $1",
        refresh_token
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|err| {
        log::error!(
            "Error occurred while fetching session with refresh_token {}: {}",
            refresh_token,
            err
        );
        Error::UnexpectedError
    })
}
