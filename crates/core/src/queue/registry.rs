use crate::queue::{Job, JobContext, JobError};
use async_trait::async_trait;
use serde_json::Value;
use std::any::type_name;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
trait JobHandler: Send + Sync {
    async fn execute(&self, payload: &Value, ctx: &JobContext) -> Result<(), JobError>;
    #[allow(dead_code)]
    fn name(&self) -> &str;
}

struct TypedJobHandler<J: Job> {
    _phantom: std::marker::PhantomData<J>,
}

impl<J: Job> TypedJobHandler<J> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<J: Job> JobHandler for TypedJobHandler<J> {
    async fn execute(&self, payload: &Value, ctx: &JobContext) -> Result<(), JobError> {
        let job: J = serde_json::from_value(payload.clone())
            .map_err(|e| JobError::Fatal(format!("Failed to deserialize job: {}", e)))?;

        job.handle(ctx).await
    }

    fn name(&self) -> &str {
        type_name::<J>()
    }
}

pub struct JobRegistry {
    handlers: HashMap<String, Arc<dyn JobHandler>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<J: Job>(&mut self) {
        let name = type_name::<J>().to_string();
        let handler = Arc::new(TypedJobHandler::<J>::new());

        tracing::debug!("Registering job type: {}", name);
        self.handlers.insert(name, handler);
    }

    pub async fn execute(&self, payload: &Value, ctx: &JobContext) -> Result<(), JobError> {
        let job_type = payload
            .get("__type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JobError::Fatal("Missing __type field in job payload".to_string()))?;

        let handler = self.handlers.get(job_type).ok_or_else(|| {
            JobError::Fatal(format!("No handler registered for job type: {}", job_type))
        })?;

        handler.execute(payload, ctx).await
    }

    pub fn has_handler(&self, job_type: &str) -> bool {
        self.handlers.contains_key(job_type)
    }

    pub fn job_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
    fn test_registry_register() {
        let mut registry = JobRegistry::new();
        assert_eq!(registry.len(), 0);

        registry.register::<TestJob>();
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_has_handler() {
        let mut registry = JobRegistry::new();
        registry.register::<TestJob>();

        let job_type = type_name::<TestJob>();
        assert!(registry.has_handler(job_type));
        assert!(!registry.has_handler("NonExistentJob"));
    }

    #[test]
    fn test_registry_job_types() {
        let mut registry = JobRegistry::new();
        registry.register::<TestJob>();

        let types = registry.job_types();
        assert_eq!(types.len(), 1);
        assert!(types[0].contains("TestJob"));
    }
}
