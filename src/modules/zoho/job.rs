use super::service;
use crate::types::{Context, SchedulableJob};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

async fn refresh_access_token_job(ctx: Arc<Context>) -> Result<(), apalis::prelude::Error> {
    service::refresh_access_token(ctx).await;
    Ok(())
}

fn setup_refresh_access_token_job(
    ctx: Arc<Context>,
) -> Arc<
    dyn Fn()
            -> Pin<Box<dyn std::future::Future<Output = Result<(), apalis::prelude::Error>> + Send>>
        + Send
        + Sync,
> {
    Arc::new(move || {
        let ctx = ctx.clone();
        Box::pin(async move { refresh_access_token_job(ctx).await })
    })
}

pub fn list(ctx: Arc<Context>) -> Vec<SchedulableJob> {
    vec![SchedulableJob {
        schedule: apalis::cron::Schedule::from_str("@hourly").expect("Couldn't create schedule"),
        job: setup_refresh_access_token_job(ctx),
    }]
}
