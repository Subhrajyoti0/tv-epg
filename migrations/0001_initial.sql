PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 100,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY,
    canonical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    tvg_id TEXT,
    language TEXT,
    country TEXT,
    group_name TEXT,
    category TEXT,
    quality TEXT,
    logo TEXT,
    stream_url TEXT,
    confidence REAL NOT NULL DEFAULT 0,
    confidence_source TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS provider_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_kind TEXT NOT NULL,
    provider_channel_id TEXT NOT NULL,
    channel_id TEXT,
    name TEXT NOT NULL,
    language TEXT,
    country TEXT,
    group_name TEXT,
    category TEXT,
    quality TEXT,
    logo TEXT,
    stream_url TEXT,
    premium INTEGER NOT NULL DEFAULT 0,
    catchup INTEGER NOT NULL DEFAULT 0,
    hidden INTEGER NOT NULL DEFAULT 0,
    raw_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(provider_kind, provider_channel_id)
);

CREATE TABLE IF NOT EXISTS programmes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_kind TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    programme_id TEXT,
    title TEXT NOT NULL,
    subtitle TEXT,
    description TEXT,
    start_time TEXT NOT NULL,
    stop_time TEXT NOT NULL,
    categories_json TEXT,
    genres_json TEXT,
    language TEXT,
    image TEXT,
    actors_json TEXT,
    directors_json TEXT,
    rating_system TEXT,
    rating_value TEXT,
    is_repeat INTEGER NOT NULL DEFAULT 0,
    is_live INTEGER NOT NULL DEFAULT 0,
    catchup INTEGER NOT NULL DEFAULT 0,
    raw_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(provider_kind, channel_id, start_time, stop_time, title)
);

CREATE TABLE IF NOT EXISTS aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alias TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    source TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(alias, canonical_name)
);

CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_provider TEXT NOT NULL,
    source_channel_id TEXT NOT NULL,
    target_provider TEXT,
    target_channel_id TEXT,
    unified_channel_id TEXT,
    score REAL NOT NULL,
    decision TEXT NOT NULL,
    confidence TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS reviews (
    id TEXT PRIMARY KEY,
    source_provider TEXT NOT NULL,
    source_channel_id TEXT NOT NULL,
    source_name TEXT NOT NULL,
    best_score REAL NOT NULL DEFAULT 0,
    reason TEXT NOT NULL,
    candidates_json TEXT,
    resolved INTEGER NOT NULL DEFAULT 0,
    resolution TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS logos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id TEXT NOT NULL,
    provider_kind TEXT,
    url TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    preferred INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    error TEXT,
    metadata_json TEXT
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    message TEXT NOT NULL,
    metadata_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_provider_channels_provider
ON provider_channels(provider_kind);

CREATE INDEX IF NOT EXISTS idx_provider_channels_name
ON provider_channels(name);

CREATE INDEX IF NOT EXISTS idx_programmes_channel
ON programmes(channel_id);

CREATE INDEX IF NOT EXISTS idx_programmes_time
ON programmes(start_time, stop_time);

CREATE INDEX IF NOT EXISTS idx_matches_source
ON matches(source_provider, source_channel_id);

CREATE INDEX IF NOT EXISTS idx_reviews_resolved
ON reviews(resolved);
