-- V001: initial schema for Joséphine (metrics, events, notifications, check log).
CREATE TABLE IF NOT EXISTS metrics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name  TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    value       REAL NOT NULL,
    recorded_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_metrics_check_time
    ON metrics(check_name, recorded_at);

CREATE TABLE IF NOT EXISTS events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name  TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    from_state  TEXT NOT NULL,
    to_state    TEXT NOT NULL,
    value       REAL NOT NULL,
    message     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notifications (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    channel  TEXT NOT NULL,
    sent_at  TEXT NOT NULL,
    FOREIGN KEY(event_id) REFERENCES events(id)
);

CREATE TABLE IF NOT EXISTS checks_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name    TEXT NOT NULL,
    status        TEXT NOT NULL,
    duration_ms   INTEGER NOT NULL,
    error_message TEXT,
    ran_at        TEXT NOT NULL
);
