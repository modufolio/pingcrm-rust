-- Create jobs table for background queue system
CREATE TABLE IF NOT EXISTS jobs (
    id BIGSERIAL PRIMARY KEY,
    queue VARCHAR(255) NOT NULL DEFAULT 'default',
    payload JSONB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    available_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    finished_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT
);

-- Index for efficient queue polling
CREATE INDEX IF NOT EXISTS idx_jobs_queue_status_available
    ON jobs(queue, status, available_at)
    WHERE status IN ('pending', 'retrying');

-- Index for job lookup
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);

-- Index for cleanup queries
CREATE INDEX IF NOT EXISTS idx_jobs_finished_at ON jobs(finished_at);
