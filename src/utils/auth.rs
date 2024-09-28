use chrono::Utc;

use crate::{
    repository::{self, session::Session},
    types,
};
use std::sync::Arc;

pub enum Error {
    UnexpectedError,
    InvalidSession,
    ExpiredToken,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn create_session(ctx: Arc<types::Context>, user_id: String) -> Result<Session> {
    repository::session::create(ctx.db_conn.clone(), user_id)
        .await
        .map_err(|_| Error::UnexpectedError)
}

pub async fn verify_access_token(
    ctx: Arc<types::Context>,
    access_token: String,
) -> Result<Session> {
    let session = repository::session::find_by_access_token(ctx.db_conn.clone(), access_token)
        .await
        .map_err(|_| Error::UnexpectedError)?
        .ok_or(Error::InvalidSession)?;

    if session.access_token_expires_at > Utc::now().naive_utc() {
        return Err(Error::ExpiredToken);
    };

    Ok(session)
}

pub async fn verify_refresh_token(
    ctx: Arc<types::Context>,
    refresh_token: String,
) -> Result<Session> {
    let session = repository::session::find_by_refresh_token(ctx.db_conn.clone(), refresh_token)
        .await
        .map_err(|_| Error::UnexpectedError)?
        .ok_or(Error::InvalidSession)?;

    if session.refresh_token_expires_at > Utc::now().naive_utc() {
        return Err(Error::ExpiredToken);
    };

    Ok(session)
}
