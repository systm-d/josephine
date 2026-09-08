use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use rusqlite::{Connection, params};

use crate::paths::Paths;
use crate::rules::StateTransition;

/// Embedded, ordered schema migrations. The version of `MIGRATIONS[i]` is `i + 1`.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/V001__init.sql")];

/// Apply every migration newer than the recorded schema version. Idempotent.
// NOTE: future multi-statement migrations should be wrapped in a transaction (BEGIN/COMMIT) so a mid-migration failure cannot leave a partially-applied, unstamped schema.
fn apply_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")?;
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    for (index, sql) in MIGRATIONS.iter().enumerate() {
        let version = index as i64 + 1;
        if version > current {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [version],
            )?;
        }
    }
    Ok(())
}

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
pub struct MetricSummary {
    pub min: f64,
    pub avg: f64,
    pub max: f64,
    /// Hourly averages over the window, chronological — for a sparkline.
    pub series: Vec<f64>,
}

impl Storage {
    pub fn open(paths: &Paths) -> Result<Self> {
        paths.ensure_dirs()?;
        let conn = Connection::open(&paths.database)
            .with_context(|| format!("ouverture de {}", paths.database.display()))?;
        apply_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn insert_metrics(&self, check_name: &str, metrics: &[(String, f64)]) -> Result<()> {
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
        self.conn
            .execute("DELETE FROM events WHERE created_at < ?1", params![cutoff])?;
        Ok(())
    }

    /// How a series is bucketed for display. An hourly point over a week gives
    /// 168 of them, which is noise in a sparkline a few dozen characters wide;
    /// past two days, a daily average reads better and still shows the shape.
    fn bucket_width(hours: i64) -> usize {
        // RFC3339 prefix lengths: "YYYY-MM-DDTHH" is 13, "YYYY-MM-DD" is 10.
        if hours <= 48 { 13 } else { 10 }
    }

    /// Min/avg/max and an averaged series for one metric over the last
    /// `hours`. Returns `None` when no sample was recorded in the window.
    pub fn metric_summary(
        &self,
        check: &str,
        metric: &str,
        hours: i64,
    ) -> Result<Option<MetricSummary>> {
        let since = (Utc::now() - Duration::hours(hours)).to_rfc3339();

        let (min, avg, max): (Option<f64>, Option<f64>, Option<f64>) = self.conn.query_row(
            "SELECT MIN(value), AVG(value), MAX(value) FROM metrics
             WHERE check_name = ?1 AND metric_name = ?2 AND recorded_at >= ?3",
            params![check, metric, &since],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let (Some(min), Some(avg), Some(max)) = (min, avg, max) else {
            return Ok(None);
        };

        // One point per bucket, keyed by the RFC3339 date(-hour) prefix.
        let width = Self::bucket_width(hours) as i64;
        let mut stmt = self.conn.prepare(
            "SELECT AVG(value) FROM metrics
             WHERE check_name = ?1 AND metric_name = ?2 AND recorded_at >= ?3
             GROUP BY substr(recorded_at, 1, ?4)
             ORDER BY substr(recorded_at, 1, ?4)",
        )?;
        let series = stmt
            .query_map(params![check, metric, &since, width], |row| {
                row.get::<_, f64>(0)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(MetricSummary {
            min,
            avg,
            max,
            series,
        }))
    }

    /// Hourly-averaged `(day, value)` points for one metric over the last
    /// `days`, oldest first. `day` is days since the Unix epoch (a consistent
    /// day-scaled x), so a linear fit over these points yields a per-day slope.
    /// Empty when nothing was recorded. Used by the forecast engine.
    pub fn metric_series_since(
        &self,
        check: &str,
        metric: &str,
        days: u32,
    ) -> Result<Vec<(f64, f64)>> {
        let since = (Utc::now() - Duration::days(i64::from(days))).to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT substr(recorded_at, 1, 13) AS hour, AVG(value) FROM metrics
             WHERE check_name = ?1 AND metric_name = ?2 AND recorded_at >= ?3
             GROUP BY hour ORDER BY hour",
        )?;
        let rows = stmt
            .query_map(params![check, metric, &since], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let points = rows
            .into_iter()
            .filter_map(|(bucket, value)| hour_bucket_to_day(&bucket).map(|day| (day, value)))
            .collect();
        Ok(points)
    }

    /// The most recent state-change events over the last 24 h (newest first).
    pub fn recent_events(&self, limit: usize) -> Result<Vec<EventRecord>> {
        self.events_since(24, limit)
    }

    /// State-change events over the last `hours` (newest first), capped at
    /// `limit`. Backs both `history` (24 h) and `report --since` (a digest).
    pub fn events_since(&self, hours: i64, limit: usize) -> Result<Vec<EventRecord>> {
        let since = (Utc::now() - Duration::hours(hours)).to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT check_name, metric_name, from_state, to_state, value, message, created_at
             FROM events WHERE created_at >= ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;

        let events = stmt
            .query_map(params![since, limit as i64], |row| {
                Ok(EventRecord {
                    check_name: row.get(0)?,
                    metric_name: row.get(1)?,
                    from_state: row.get(2)?,
                    to_state: row.get(3)?,
                    value: row.get(4)?,
                    message: row.get(5)?,
                    created_at: row
                        .get::<_, String>(6)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(events)
    }
}

/// Convert an `"YYYY-MM-DDTHH"` hour bucket into days since the Unix epoch.
fn hour_bucket_to_day(bucket: &str) -> Option<f64> {
    let stamp =
        NaiveDateTime::parse_from_str(&format!("{bucket}:00:00"), "%Y-%m-%dT%H:%M:%S").ok()?;
    Some(stamp.and_utc().timestamp() as f64 / 86_400.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A migrated, empty database that never touches the disk.
    fn in_memory() -> Storage {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        Storage { conn }
    }

    #[test]
    fn hour_bucket_parses_to_a_day_offset() {
        // 1970-01-02T00 is exactly one day after the epoch.
        assert_eq!(hour_bucket_to_day("1970-01-02T00"), Some(1.0));
        // Consecutive hours are 1/24 of a day apart.
        let a = hour_bucket_to_day("2026-07-28T10").unwrap();
        let b = hour_bucket_to_day("2026-07-28T11").unwrap();
        assert!((b - a - 1.0 / 24.0).abs() < 1e-9);
        assert!(hour_bucket_to_day("not-a-date").is_none());
    }

    #[test]
    fn buckets_are_hourly_up_to_two_days_and_daily_beyond() {
        // "YYYY-MM-DDTHH" = 13 chars (hourly), "YYYY-MM-DD" = 10 (daily).
        assert_eq!(Storage::bucket_width(6), 13);
        assert_eq!(Storage::bucket_width(24), 13);
        assert_eq!(Storage::bucket_width(48), 13);
        // A week of hourly points is 168 of them — noise in a sparkline.
        assert_eq!(Storage::bucket_width(49), 10);
        assert_eq!(Storage::bucket_width(168), 10);
    }

    #[test]
    fn summary_covers_the_window_and_averages_each_bucket() {
        let storage = in_memory();

        // Two samples inside the same hour, one an hour later, and one well
        // outside a 6 h window.
        let now = Utc::now();
        for (offset_hours, value) in [(0.0, 10.0), (0.1, 30.0), (1.0, 50.0), (30.0, 999.0)] {
            let at = (now - Duration::minutes((offset_hours * 60.0) as i64)).to_rfc3339();
            storage
                .conn
                .execute(
                    "INSERT INTO metrics (check_name, metric_name, value, recorded_at)
                     VALUES ('cpu', 'usage_percent', ?1, ?2)",
                    params![value, at],
                )
                .unwrap();
        }

        let summary = storage
            .metric_summary("cpu", "usage_percent", 6)
            .unwrap()
            .expect("samples inside the window");

        // The 999 sits 30 h back and must not reach a 6 h summary.
        assert_eq!(summary.max, 50.0);
        assert_eq!(summary.min, 10.0);
        assert_eq!(summary.avg, 30.0);

        // Two hourly buckets: (10 + 30) / 2 and 50 — the pair recorded in the
        // same hour is averaged into one point rather than plotted twice.
        assert_eq!(summary.series.len(), 2);
        assert!(summary.series.contains(&20.0), "{:?}", summary.series);
        assert!(summary.series.contains(&50.0), "{:?}", summary.series);
    }

    #[test]
    fn summary_is_none_when_the_window_holds_nothing() {
        let storage = in_memory();
        assert!(
            storage
                .metric_summary("cpu", "usage_percent", 24)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn migrations_apply_to_in_memory_database() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        let applied: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }

    #[test]
    fn applying_migrations_twice_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        apply_migrations(&conn).unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, MIGRATIONS.len() as i64);
    }
}
