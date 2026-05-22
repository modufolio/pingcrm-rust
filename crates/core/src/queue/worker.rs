use crate::queue::{JobContext, JobError, JobRecord, JobRegistry, Queue, QueueError};
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[derive(Clone)]
pub struct WorkerConfig {
    pub queue_name: String,

    pub concurrency: usize,

    pub shutdown_timeout: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queue_name: "default".to_string(),
            concurrency: 10,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

pub struct Worker {
    queue: Arc<Queue>,
    registry: Arc<JobRegistry>,
    config: WorkerConfig,
}

impl Worker {
    pub fn new(queue: Queue, registry: JobRegistry, config: WorkerConfig) -> Self {
        Self {
            queue: Arc::new(queue),
            registry: Arc::new(registry),
            config,
        }
    }

    pub async fn run(self: Arc<Self>) -> Result<(), QueueError> {
        tracing::info!(
            "Starting worker for queue '{}' with concurrency {}",
            self.config.queue_name,
            self.config.concurrency
        );

        let job_stream = self.queue.stream(&self.config.queue_name);

        job_stream
            .for_each_concurrent(self.config.concurrency, |job| {
                let worker = Arc::clone(&self);
                async move {
                    if let Err(e) = worker.process_job(job).await {
                        tracing::error!("Error processing job: {}", e);
                    }
                }
            })
            .await;

        Ok(())
    }

    async fn process_job(&self, job: JobRecord) -> Result<(), QueueError> {
        tracing::info!("Processing job {} (attempt {})", job.id, job.attempts + 1);

        let ctx = JobContext::new(job.id, job.attempts as u32);

        let job_timeout = Duration::from_secs(300);
        let result = timeout(job_timeout, self.registry.execute(&job.payload, &ctx)).await;

        match result {
            Ok(Ok(())) => {
                tracing::info!("Job {} completed successfully", job.id);
                self.queue.mark_completed(job.id).await?;
            }
            Ok(Err(job_error)) => {
                tracing::error!("Job {} failed: {}", job.id, job_error);
                self.queue.mark_failed(job.id, &job_error).await?;
            }
            Err(_) => {
                tracing::error!("Job {} timed out", job.id);
                self.queue.mark_failed(job.id, &JobError::Timeout).await?;
            }
        }

        Ok(())
    }

    pub async fn shutdown(&self) {
        tracing::info!("Shutting down worker gracefully...");
        tokio::time::sleep(self.config.shutdown_timeout).await;
        tracing::info!("Worker shutdown complete");
    }
}

pub async fn run_worker(worker: Worker) {
    let worker = Arc::new(worker);

    let worker_clone = Arc::clone(&worker);
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker_clone.run().await {
            tracing::error!("Worker error: {}", e);
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");

    tracing::info!("Received shutdown signal");

    worker.shutdown().await;

    let _ = tokio::time::timeout(Duration::from_secs(30), worker_handle).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.queue_name, "default");
        assert_eq!(config.concurrency, 10);
    }
}
