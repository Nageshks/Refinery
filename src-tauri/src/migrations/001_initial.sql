-- Refinery initial schema

CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    format_type TEXT NOT NULL DEFAULT 'markdown',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    current_version_id TEXT,
    last_review_session_id TEXT,
    local_status TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS review_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    page_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    prompt_version TEXT NOT NULL,
    raw_response TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS suggestion_groups (
    id TEXT PRIMARY KEY NOT NULL,
    page_id TEXT NOT NULL,
    review_session_id TEXT NOT NULL,
    category TEXT NOT NULL,
    label TEXT NOT NULL,
    approval_state TEXT NOT NULL DEFAULT 'pending',
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE,
    FOREIGN KEY (review_session_id) REFERENCES review_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS suggestion_items (
    id TEXT PRIMARY KEY NOT NULL,
    group_id TEXT NOT NULL,
    original_text TEXT NOT NULL,
    replacement_text TEXT NOT NULL,
    span_start INTEGER NOT NULL,
    span_end INTEGER NOT NULL,
    explanation TEXT NOT NULL DEFAULT '',
    confidence REAL NOT NULL DEFAULT 0.5,
    approval_state TEXT NOT NULL DEFAULT 'pending',
    conflict_state TEXT,
    FOREIGN KEY (group_id) REFERENCES suggestion_groups(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS version_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    page_id TEXT NOT NULL,
    content TEXT NOT NULL,
    source_review_session_id TEXT,
    applied_suggestion_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS provider_configs (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL DEFAULT 'openrouter',
    endpoint TEXT,
    selected_model TEXT NOT NULL DEFAULT 'openai/gpt-4o',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_pages_status ON pages(local_status);
CREATE INDEX IF NOT EXISTS idx_review_sessions_page ON review_sessions(page_id);
CREATE INDEX IF NOT EXISTS idx_suggestion_groups_session ON suggestion_groups(review_session_id);
CREATE INDEX IF NOT EXISTS idx_suggestion_items_group ON suggestion_items(group_id);
CREATE INDEX IF NOT EXISTS idx_version_snapshots_page ON version_snapshots(page_id);
CREATE INDEX IF NOT EXISTS idx_provider_configs_enabled ON provider_configs(enabled);
