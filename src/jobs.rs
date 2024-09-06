use crate::types::SchedulableJob;
use std::sync::Arc;

use apalis::cron::CronStream;
use apalis::prelude::*;
use apalis::utils::TokioExecutor;

use crate::{types, utils};

pub async fn monitor(ctx: Arc<types::Context>) -> apalis::prelude::Monitor<TokioExecutor> {
    let all_jobs = utils::notification::email::jobs::list(ctx).await;

    let storage = types::JobStorage::new();
    let mut monitor = apalis::prelude::Monitor::<TokioExecutor>::new();

    for job in all_jobs {
        let job_clone = job.clone();
        let worker = WorkerBuilder::new("crate::utils::notification::email::jobs::refresh_token")
            .with_storage(storage.clone())
            .stream(CronStream::new(job.schedule()).into_stream())
            .build_fn(move |j: types::Job| {
                let job_clone = job_clone.clone();
                async move { job_clone.run().await }
            });
        monitor = monitor.register_with_count(1, worker);
    }

    monitor
}
