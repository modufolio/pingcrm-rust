#[derive(Clone)]
pub struct JobContext {
    pub job_id: i64,

    pub attempt: u32,
}

impl JobContext {
    pub fn new(job_id: i64, attempt: u32) -> Self {
        Self { job_id, attempt }
    }
}
