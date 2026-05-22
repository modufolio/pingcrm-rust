use crate::queue::{Job, JobError, JobId, JobRecord, QueueBackend, QueueError};
use chrono::Utc;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use std::time::Duration;

pub struct Queue {
    backend: Arc<dyn QueueBackend>,
}

impl Queue {
    pub fn new(backend: Arc<dyn QueueBackend>) -> Self {
        Self { backend }
    }

    pub async fn dispatch<J: Job>(&self, job: J) -> Result<JobId, QueueError> {
        let queue_name = job.queue_name();
        let max_attempts = job.max_retries() as i32 + 1;
        let payload = serde_json::to_value(&job)?;
        let available_at = Utc::now();

        self.backend
            .push(queue_name, payload, max_attempts, available_at)
            .await
    }

    pub async fn dispatch_delayed<J: Job>(
        &self,
        job: J,
        delay: Duration,
    ) -> Result<JobId, QueueError> {
        let queue_name = job.queue_name();
        let max_attempts = job.max_retries() as i32 + 1;
        let payload = serde_json::to_value(&job)?;
        let available_at = Utc::now() + chrono::Duration::from_std(delay).unwrap();

        self.backend
            .push(queue_name, payload, max_attempts, available_at)
            .await
    }

    pub fn stream(&self, queue: &str) -> impl Stream<Item = JobRecord> + '_ {
        let queue = queue.to_string();

        async_stream::stream! {
            loop {
                match self.backend.pop(&queue).await {
                    Ok(Some(job)) => {
                        tracing::debug!("Popped job {} from queue {}", job.id, queue);
                        yield job;
                    }
                    Ok(None) => {

                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(e) => {
                        tracing::error!("Error polling queue {}: {}", queue, e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    pub fn stream_take(&self, queue: &str, n: usize) -> impl Stream<Item = JobRecord> + '_ {
        self.stream(queue).take(n)
    }

    pub async fn mark_completed(&self, job_id: JobId) -> Result<(), QueueError> {
        self.backend.mark_completed(job_id).await
    }

    pub async fn mark_failed(&self, job_id: JobId, error: &JobError) -> Result<(), QueueError> {
        let error_msg = error.to_string();

        let retry_at = if matches!(error, JobError::Retriable(_)) {
            Some(Utc::now() + chrono::Duration::seconds(60))
        } else {
            None
        };

        self.backend.mark_failed(job_id, &error_msg, retry_at).await
    }

    pub fn backend(&self) -> &Arc<dyn QueueBackend> {
        &self.backend
    }
}
