-- Create image_jobs table for thumbnail generation tracking
CREATE TABLE IF NOT EXISTS image_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    disk VARCHAR(50) NOT NULL DEFAULT 'default',
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    options TEXT NOT NULL, -- JSON encoded options
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, processing, processed, failed
    processed_at TIMESTAMP NULL,
    accessed_at TIMESTAMP NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for quick lookups by filename and disk
CREATE INDEX IF NOT EXISTS idx_image_jobs_filename_disk
    ON image_jobs(filename, disk);

-- Index for status queries
CREATE INDEX IF NOT EXISTS idx_image_jobs_status ON image_jobs(status);

-- Index for cleanup queries (find stale jobs)
CREATE INDEX IF NOT EXISTS idx_image_jobs_accessed_at ON image_jobs(accessed_at);
