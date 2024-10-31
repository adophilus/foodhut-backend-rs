use crate::types::{Context, Job, JobStorage, SchedulableJob};
use crate::utils;
use std::sync::Arc;

use apalis::cron::CronStream;
use apalis::layers::retry::{RetryLayer, RetryPolicy};
use apalis::prelude::*;
use apalis::utils::TokioExecutor;

pub async fn monitor(ctx: Arc<Context>) -> apalis::prelude::Monitor<TokioExecutor> {
    let mut all_jobs: Vec<SchedulableJob> = vec![];
    all_jobs.append(&mut utils::notification::email::jobs::list(ctx.clone()));
    all_jobs.append(&mut utils::ads::jobs::list(ctx));

    let storage = JobStorage::new();
    let mut monitor = apalis::prelude::Monitor::<TokioExecutor>::new();

    for job in all_jobs {
        // let job_clone = job.job.clone();
        (job.job)().await;
        let worker = WorkerBuilder::new("crate::utils::notification::email::jobs::refresh_token")
            .with_storage(storage.clone())
            .stream(CronStream::new(job.schedule).into_stream())
            .layer(RetryLayer::new(RetryPolicy::retries(3)))
            .build_fn(move |j: Job| {
                let job_clone = job.job.clone();
                async move { (job_clone)().await }
            });
        monitor = monitor.register_with_count(1, worker);
    }

    monitor
}
