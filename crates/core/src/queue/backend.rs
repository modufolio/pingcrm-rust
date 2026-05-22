use crate::queue::{JobId, JobRecord, JobStatus, QueueError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait QueueBackend: Send + Sync {
    async fn push(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_attempts: i32,
        available_at: DateTime<Utc>,
    ) -> Result<JobId, QueueError>;

    async fn pop(&self, queue: &str) -> Result<Option<JobRecord>, QueueError>;

    async fn mark_completed(&self, job_id: JobId) -> Result<(), QueueError>;

    async fn mark_failed(
        &self,
        job_id: JobId,
        error: &str,
        retry_at: Option<DateTime<Utc>>,
    ) -> Result<(), QueueError>;

    async fn get_job(&self, job_id: JobId) -> Result<Option<JobRecord>, QueueError>;

    async fn get_jobs_by_status(
        &self,
        status: JobStatus,
        limit: i32,
    ) -> Result<Vec<JobRecord>, QueueError>;

    async fn delete_job(&self, job_id: JobId) -> Result<(), QueueError>;

    async fn stats(&self, queue: &str) -> Result<QueueStats, QueueError>;
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total_jobs: i64,
    pub pending_jobs: i64,
    pub running_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
}

pub struct PostgresBackend;

impl PostgresBackend {
    pub fn new() -> Self {
        Self
    }
}
