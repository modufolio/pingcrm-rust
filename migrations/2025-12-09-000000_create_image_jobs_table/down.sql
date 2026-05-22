-- Drop indexes first
DROP INDEX IF EXISTS idx_image_jobs_accessed_at;
DROP INDEX IF EXISTS idx_image_jobs_status;
DROP INDEX IF EXISTS idx_image_jobs_filename_disk;

-- Drop the table
DROP TABLE IF EXISTS image_jobs;
