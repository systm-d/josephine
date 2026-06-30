use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};

use crate::paths::Paths;
use crate::rules::StateTransition;

pub struct Storage {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct MetricRecord {
    pub check_name: String,
    pub metric_name: String,
    pub value: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub check_name: String,
    pub metric_name: String,
    pub from_state: String,
    pub to_state: String,
    pub value: f64,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HistorySummary {
    pub cpu_max: Option<f64>,
    pub memory_max: Option<f64>,
    pub disk_max: Option<f64>,
    pub temperature_max: Option<f64>,
    pub recent_events: Vec<EventRecord>,
}

impl Storage {
    pub fn open(paths: &Paths) -> Result<Self> {
        paths.ensure_dirs()?;
        let conn = Connection::open(&paths.database)
            .with_context(|| format!("ouverture de {}", paths.database.display()))?;
        let storage = Self { conn };
        storage.migrate()?;
        Ok(storage)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                check_name TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                value REAL NOT NULL,
                recorded_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_metrics_check_time
                ON metrics(check_name, recorded_at);

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                check_name TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                from_state TEXT NOT NULL,
                to_state TEXT NOT NULL,
                value REAL NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                channel TEXT NOT NULL,
                sent_at TEXT NOT NULL,
                FOREIGN KEY(event_id) REFERENCES events(id)
            );

            CREATE TABLE IF NOT EXISTS checks_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                check_name TEXT NOT NULL,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                error_message TEXT,
                ran_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn insert_metrics(
        &self,
        check_name: &str,
        metrics: &[(String, f64)],
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            "INSERT INTO metrics (check_name, metric_name, value, recorded_at) VALUES (?1, ?2, ?3, ?4)",
        )?;

        for (name, value) in metrics {
            stmt.execute(params![check_name, name, value, now])?;
        }
        Ok(())
    }

    pub fn insert_event(&self, transition: &StateTransition) -> Result<i64> {
        let to_state = if transition.recovered {
            "RECOVERED".to_string()
        } else {
            transition.to.as_str().to_string()
        };

        self.conn.execute(
            "INSERT INTO events (check_name, metric_name, from_state, to_state, value, message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                transition.check_name,
                transition.metric_name,
                transition.from.as_str(),
                to_state,
                transition.value,
                transition.message,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_notification(&self, event_id: i64, channel: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO notifications (event_id, channel, sent_at) VALUES (?1, ?2, ?3)",
            params![event_id, channel, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn log_check_run(
        &self,
        check_name: &str,
        ok: bool,
        duration_ms: u64,
        error: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO checks_log (check_name, status, duration_ms, error_message, ran_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                check_name,
                if ok { "ok" } else { "error" },
                duration_ms as i64,
                error,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn purge_older_than(&self, days: u32) -> Result<()> {
        let cutoff = (Utc::now() - Duration::days(days as i64)).to_rfc3339();
        self.conn.execute(
            "DELETE FROM metrics WHERE recorded_at < ?1",
            params![cutoff],
        )?;
        self.conn.execute(
            "DELETE FROM events WHERE created_at < ?1",
            params![cutoff],
        )?;
        Ok(())
    }

    pub fn history_last_24h(&self) -> Result<HistorySummary> {
        let since = (Utc::now() - Duration::hours(24)).to_rfc3339();

        let cpu_max = self.max_metric("cpu", "usage_percent", &since)?;
        let memory_max = self.max_metric("memory", "usage_percent", &since)?;
        let disk_max = self.max_metric("disk", "usage_percent_worst", &since)?;
        let temperature_max = self.max_metric("temperature", "temp_max_celsius", &since)?;

        let mut stmt = self.conn.prepare(
            "SELECT check_name, metric_name, from_state, to_state, value, message, created_at
             FROM events WHERE created_at >= ?1 ORDER BY created_at DESC LIMIT 10",
        )?;

        let events = stmt
            .query_map(params![since], |row| {
                Ok(EventRecord {
                    check_name: row.get(0)?,
                    metric_name: row.get(1)?,
                    from_state: row.get(2)?,
                    to_state: row.get(3)?,
                    value: row.get(4)?,
                    message: row.get(5)?,
                    created_at: row.get::<_, String>(6)?.parse().unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(HistorySummary {
            cpu_max,
            memory_max,
            disk_max,
            temperature_max,
            recent_events: events,
        })
    }

    fn max_metric(&self, check: &str, metric: &str, since: &str) -> Result<Option<f64>> {
        let value: Option<f64> = self.conn.query_row(
            "SELECT MAX(value) FROM metrics
             WHERE check_name = ?1 AND metric_name = ?2 AND recorded_at >= ?3",
            params![check, metric, since],
            |row| row.get(0),
        )?;
        Ok(value)
    }
}
