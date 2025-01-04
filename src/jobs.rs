use crate::modules::ad;
use crate::types::{Context, Job, JobStorage, SchedulableJob};
use apalis::cron::CronStream;
use apalis::layers::retry::{RetryLayer, RetryPolicy};
use apalis::prelude::*;
use apalis::utils::TokioExecutor;
use std::sync::Arc;

pub async fn monitor(ctx: Arc<Context>) -> apalis::prelude::Monitor<TokioExecutor> {
    let mut all_jobs: Vec<SchedulableJob> = vec![];
    all_jobs.append(&mut ad::job::list(ctx));

    let storage = JobStorage::new();
    let mut monitor = apalis::prelude::Monitor::<TokioExecutor>::new();

    for job in all_jobs {
        // let job_clone = job.job.clone();
        if let Err(err) = (job.job)().await {
            tracing::error!("Error running job: {:?}", err);
        }
        let worker = WorkerBuilder::new("worker")
            .with_storage(storage.clone())
            .stream(CronStream::new(job.schedule).into_stream())
            .layer(RetryLayer::new(RetryPolicy::retries(3)))
            .build_fn(move |_: Job| {
                let job_clone = job.job.clone();
                async move { (job_clone)().await }
            });
        monitor = monitor.register_with_count(1, worker);
    }

    monitor
}
