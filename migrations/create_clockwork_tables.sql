-- Migration: Create Clockwork debug tables
-- Similar to PHP's Clockwork integration for debugging

-- Clockwork requests table
CREATE TABLE IF NOT EXISTS clockwork_requests (
    id TEXT PRIMARY KEY NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    request_type TEXT NOT NULL DEFAULT 'request',
    time REAL NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    uri TEXT NOT NULL,
    headers TEXT,
    get_data TEXT,
    post_data TEXT,
    cookies TEXT,
    response_status INTEGER NOT NULL DEFAULT 200,
    response_duration REAL NOT NULL DEFAULT 0.0,
    memory_usage INTEGER NOT NULL DEFAULT 0,
    queries_count INTEGER NOT NULL DEFAULT 0,
    queries_duration REAL NOT NULL DEFAULT 0.0,
    slow_queries INTEGER NOT NULL DEFAULT 0,
    middleware TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index on time for efficient queries
CREATE INDEX IF NOT EXISTS idx_clockwork_requests_time ON clockwork_requests(time DESC);

-- Clockwork queries table
CREATE TABLE IF NOT EXISTS clockwork_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    sql TEXT NOT NULL,
    bindings TEXT,
    duration REAL NOT NULL DEFAULT 0.0,
    query_type TEXT NOT NULL DEFAULT 'OTHER',
    FOREIGN KEY (request_id) REFERENCES clockwork_requests(id) ON DELETE CASCADE
);

-- Index on request_id for efficient lookups
CREATE INDEX IF NOT EXISTS idx_clockwork_queries_request_id ON clockwork_queries(request_id);
