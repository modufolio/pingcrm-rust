-- Create media table for TUS uploads
CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    width INTEGER,
    height INTEGER,
    metadata TEXT,  -- JSON string
    uploaded_by TEXT,  -- user_id (foreign key to users.id)
    title TEXT,
    alt_text TEXT,
    caption TEXT,
    is_public INTEGER NOT NULL DEFAULT 1,  -- SQLite uses INTEGER for boolean
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on filename for faster lookups
CREATE INDEX IF NOT EXISTS idx_media_filename ON media(filename);

-- Create index on uploaded_by for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_media_uploaded_by ON media(uploaded_by);

-- Create index on created_at for ordering
CREATE INDEX IF NOT EXISTS idx_media_created_at ON media(created_at DESC);
