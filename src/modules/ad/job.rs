use crate::types::{Context, SchedulableJob};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use super::repository;

async fn clean_ads_job(ctx: Arc<Context>) -> Result<(), apalis::prelude::Error> {
    tracing::info!("Cleaning up ads");

    let _ = repository::delete_expired(&ctx.db_conn.pool).await;

    Ok(())
}

fn setup_clean_ads_job(
    ctx: Arc<Context>,
) -> Arc<
    dyn Fn()
            -> Pin<Box<dyn std::future::Future<Output = Result<(), apalis::prelude::Error>> + Send>>
        + Send
        + Sync,
> {
    Arc::new(move || {
        let ctx = ctx.clone();
        Box::pin(async move { clean_ads_job(ctx).await })
    })
}

pub fn list(ctx: Arc<Context>) -> Vec<SchedulableJob> {
    vec![SchedulableJob {
        schedule: apalis::cron::Schedule::from_str("0 * * * * *")
            .expect("Couldn't create schedule!"),
        job: setup_clean_ads_job(ctx),
    }]
}
