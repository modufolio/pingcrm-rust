use std::fmt;

#[derive(Debug)]
pub enum JobError {
    Retriable(String),

    Fatal(String),

    Timeout,
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobError::Retriable(msg) => write!(f, "Retriable error: {}", msg),
            JobError::Fatal(msg) => write!(f, "Fatal error: {}", msg),
            JobError::Timeout => write!(f, "Job timed out"),
        }
    }
}

impl std::error::Error for JobError {}

#[derive(Debug)]
pub enum QueueError {
    Database(String),

    Serialization(String),

    NotFound(i64),

    InvalidState(String),
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::Database(msg) => write!(f, "Database error: {}", msg),
            QueueError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            QueueError::NotFound(id) => write!(f, "Job {} not found", id),
            QueueError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for QueueError {}

impl From<serde_json::Error> for QueueError {
    fn from(err: serde_json::Error) -> Self {
        QueueError::Serialization(err.to_string())
    }
}
