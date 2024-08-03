use super::super::{Notification, Result};
use std::sync::Arc;
use crate::types;

pub async fn send(ctx: Arc<types::Context>, notification: Notification) -> Result<()> {
    Ok(())
}
