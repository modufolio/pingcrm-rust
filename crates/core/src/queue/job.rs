use crate::queue::{JobContext, JobError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type JobId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,

    Running,

    Completed,

    Failed,

    Retrying,
}

#[async_trait]
pub trait Job: Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static {
    async fn handle(&self, ctx: &JobContext) -> Result<(), JobError>;

    fn queue_name(&self) -> &'static str {
        "default"
    }

    fn max_retries(&self) -> u32 {
        3
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(300)
    }

    fn retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = 10;
        let delay_secs = base_delay * 10_u64.pow(attempt);
        Duration::from_secs(delay_secs.min(3600))
    }

    fn name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub id: JobId,
    pub queue: String,
    pub payload: serde_json::Value,
    pub attempts: i32,
    pub max_attempts: i32,
    pub status: JobStatus,
    pub available_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct TestJob {
        value: i32,
    }

    #[async_trait]
    impl Job for TestJob {
        async fn handle(&self, _ctx: &JobContext) -> Result<(), JobError> {
            Ok(())
        }
    }

    #[test]
    fn test_job_defaults() {
        let job = TestJob { value: 42 };
        assert_eq!(job.queue_name(), "default");
        assert_eq!(job.max_retries(), 3);
        assert_eq!(job.timeout(), Duration::from_secs(300));
    }

    #[test]
    fn test_retry_delay() {
        let job = TestJob { value: 42 };
        assert_eq!(job.retry_delay(0), Duration::from_secs(10));
        assert_eq!(job.retry_delay(1), Duration::from_secs(100));
        assert_eq!(job.retry_delay(2), Duration::from_secs(1000));

        assert_eq!(job.retry_delay(3), Duration::from_secs(3600));
    }
}
