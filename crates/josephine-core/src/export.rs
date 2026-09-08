//! Prometheus textfile export — the local, opt-in way to pull Joséphine's
//! latest results into a dashboard.
//!
//! No server and no port: the daemon writes a `.prom` file that
//! node-exporter's textfile collector already knows how to read. A homelab
//! that graphs anything at all has node-exporter; a guardian that opens a
//! listening socket has a new attack surface. So Joséphine writes a file, and
//! nothing about her becomes reachable from the network.
//!
//! Off by default. When enabled, the file holds only what `josephine status`
//! already shows: per-check metric values and a severity.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::check::{CheckResult, Severity};

/// The latest result per check, ready to render.
///
/// A `BTreeMap` so the file's line order is stable between writes — a diff of
/// two exports should show what changed, not a reshuffle.
#[derive(Debug, Default)]
pub struct PrometheusExport {
    latest: BTreeMap<String, CheckSnapshot>,
}

#[derive(Debug)]
struct CheckSnapshot {
    severity: Severity,
    /// `(metric name, unit, value)`.
    metrics: Vec<(String, String, f64)>,
}

impl PrometheusExport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a check's newest result, replacing whatever it last reported.
    pub fn record(&mut self, result: &CheckResult) {
        self.latest.insert(
            result.check_name.clone(),
            CheckSnapshot {
                severity: result.worst_severity(),
                metrics: result
                    .metrics
                    .iter()
                    .map(|m| (m.name.clone(), m.unit.clone(), m.value))
                    .collect(),
            },
        );
    }

    /// Render the exposition text.
    pub fn render(&self, written_at_unix_secs: i64) -> String {
        let mut out = String::new();

        out.push_str("# HELP josephine_metric Latest value reported by a Joséphine check.\n");
        out.push_str("# TYPE josephine_metric gauge\n");
        for (check, snapshot) in &self.latest {
            for (metric, unit, value) in &snapshot.metrics {
                out.push_str(&format!(
                    "josephine_metric{{check=\"{}\",metric=\"{}\",unit=\"{}\"}} {}\n",
                    escape(check),
                    escape(metric),
                    escape(unit),
                    render_value(*value),
                ));
            }
        }

        out.push_str(
            "# HELP josephine_check_severity Worst severity for a check: 0 ok, 1 attention, 2 critical.\n",
        );
        out.push_str("# TYPE josephine_check_severity gauge\n");
        for (check, snapshot) in &self.latest {
            let severity = match snapshot.severity {
                Severity::Info => 0,
                Severity::Attention => 1,
                Severity::Critique => 2,
            };
            out.push_str(&format!(
                "josephine_check_severity{{check=\"{}\"}} {severity}\n",
                escape(check),
            ));
        }

        out.push_str(
            "# HELP josephine_last_write_timestamp_seconds When Joséphine last wrote this file.\n",
        );
        out.push_str("# TYPE josephine_last_write_timestamp_seconds gauge\n");
        out.push_str(&format!(
            "josephine_last_write_timestamp_seconds {written_at_unix_secs}\n"
        ));

        out
    }

    /// Write the exposition text to `path`, atomically.
    ///
    /// node-exporter reads the file on its own schedule, so a half-written one
    /// would be scraped as truncated. Writing a sibling temporary file and
    /// renaming it means a reader sees either the previous export or the new
    /// one, never a partial line.
    pub fn write_to(&self, path: &Path, written_at_unix_secs: i64) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }

        let temp = temp_path(path);
        {
            let mut file = std::fs::File::create(&temp)
                .with_context(|| format!("creating {}", temp.display()))?;
            file.write_all(self.render(written_at_unix_secs).as_bytes())
                .with_context(|| format!("writing {}", temp.display()))?;
            file.sync_all().ok();
        }
        std::fs::rename(&temp, path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }
}

/// `foo.prom` → `foo.prom.tmp`, alongside the target so the rename stays on
/// one filesystem (a rename across filesystems is not atomic, and fails).
fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Escape a Prometheus label value: backslash, double quote, newline.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

/// Render a sample value. Prometheus has no notion of NaN/∞ in a textfile that
/// a collector will accept happily, so a value that is not finite is written
/// as the exposition format's own `NaN`.
fn render_value(value: f64) -> String {
    if value.is_finite() {
        format!("{value}")
    } else if value.is_nan() {
        "NaN".to_string()
    } else if value > 0.0 {
        "+Inf".to_string()
    } else {
        "-Inf".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::Metric;

    fn result(check: &str, metric: &str, value: f64, warning: Option<f64>) -> CheckResult {
        CheckResult {
            check_name: check.into(),
            metrics: vec![Metric {
                name: metric.into(),
                value,
                unit: "%".into(),
                threshold_warning: warning,
                threshold_critical: None,
            }],
            details: vec![],
            top_processes: vec![],
            status_value: None,
        }
    }

    #[test]
    fn renders_values_and_severity() {
        let mut export = PrometheusExport::new();
        export.record(&result("cpu", "usage_percent", 12.5, Some(85.0)));
        export.record(&result("disk", "usage_percent_worst", 91.0, Some(85.0)));

        let text = export.render(1_700_000_000);

        assert!(
            text.contains(
                "josephine_metric{check=\"cpu\",metric=\"usage_percent\",unit=\"%\"} 12.5"
            ),
            "{text}"
        );
        assert!(
            text.contains("josephine_check_severity{check=\"cpu\"} 0"),
            "{text}"
        );
        // disk is over its warning threshold.
        assert!(
            text.contains("josephine_check_severity{check=\"disk\"} 1"),
            "{text}"
        );
        assert!(
            text.contains("josephine_last_write_timestamp_seconds 1700000000"),
            "{text}"
        );
    }

    #[test]
    fn every_metric_family_is_declared_once() {
        let mut export = PrometheusExport::new();
        export.record(&result("cpu", "usage_percent", 1.0, None));
        export.record(&result("memory", "usage_percent", 2.0, None));

        let text = export.render(0);

        // A repeated HELP/TYPE for the same family makes a scrape fail, so it
        // must be emitted per family, not per sample.
        assert_eq!(text.matches("# TYPE josephine_metric gauge").count(), 1);
        assert_eq!(
            text.matches("# TYPE josephine_check_severity gauge")
                .count(),
            1
        );
    }

    #[test]
    fn a_later_result_replaces_the_earlier_one() {
        let mut export = PrometheusExport::new();
        export.record(&result("cpu", "usage_percent", 10.0, None));
        export.record(&result("cpu", "usage_percent", 90.0, None));

        let text = export.render(0);
        assert_eq!(text.matches("josephine_metric{check=\"cpu\"").count(), 1);
        assert!(text.contains("} 90"), "{text}");
    }

    #[test]
    fn label_values_are_escaped() {
        let mut export = PrometheusExport::new();
        export.record(&result("od\"d", "back\\slash", 1.0, None));

        let text = export.render(0);
        assert!(text.contains("check=\"od\\\"d\""), "{text}");
        assert!(text.contains("metric=\"back\\\\slash\""), "{text}");
    }

    #[test]
    fn writing_is_atomic_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join("josephine-export-test");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("nested/josephine.prom");

        let mut export = PrometheusExport::new();
        export.record(&result("cpu", "usage_percent", 5.0, None));
        export.write_to(&path, 42).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(
            written.contains("josephine_metric{check=\"cpu\""),
            "{written}"
        );
        assert!(
            !path.with_extension("prom.tmp").exists(),
            "the temporary file survived the rename"
        );

        // A second write replaces the first rather than appending to it.
        export.record(&result("cpu", "usage_percent", 6.0, None));
        export.write_to(&path, 43).unwrap();
        let rewritten = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            rewritten.matches("josephine_metric{check=\"cpu\"").count(),
            1
        );
        assert!(rewritten.contains("} 6"), "{rewritten}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
