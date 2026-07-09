//! Machine-readable `--json` rendering for `status`, `doctor`, and `report`.
//!
//! Prints a single pretty-JSON array to stdout — no colour, no header, no
//! progress noise. Callers must collect results via a quiet path (no stdout
//! writes) before calling this.

use josephine_core::check::{CheckResult, Metric, Severity};
use serde::Serialize;

#[derive(Serialize)]
struct JsonCheck<'a> {
    check: &'a str,
    severity: Severity,
    value: Option<&'a str>,
    details: &'a [String],
    metrics: &'a [Metric],
}

/// Print the checks as a single pretty-JSON document to stdout.
pub fn print_checks(results: &[CheckResult]) {
    let checks: Vec<JsonCheck> = results
        .iter()
        .map(|r| JsonCheck {
            check: &r.check_name,
            severity: r.worst_severity(),
            value: r.status_value.as_deref(),
            details: &r.details,
            metrics: &r.metrics,
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&checks).expect("serialize checks")
    );
}
