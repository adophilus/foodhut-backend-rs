use super::service;
use crate::types::{Context, SchedulableJob};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

async fn bank_fetch_job(ctx: Arc<Context>) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Fetching banks from paystack...");
    service::update_paystack_banks(ctx).await;
    Ok(())
}

fn setup_bank_fetch_job(
    ctx: Arc<Context>,
) -> Arc<
    dyn Fn()
            -> Pin<Box<dyn std::future::Future<Output = Result<(), apalis::prelude::Error>> + Send>>
        + Send
        + Sync,
> {
    Arc::new(move || {
        let ctx = ctx.clone();
        Box::pin(async move { bank_fetch_job(ctx).await })
    })
}

pub fn list(ctx: Arc<Context>) -> Vec<SchedulableJob> {
    vec![SchedulableJob {
        schedule: apalis::cron::Schedule::from_str("@daily").expect("Couldn't create schedule"),
        job: setup_bank_fetch_job(ctx),
    }]
}
