//! Kernel incidents — scans the recent kernel journal for out-of-memory kills
//! and faults (oops / BUG / panic): the quiet events that destabilise a machine.
//! Reads `journalctl -k`; degrades gracefully if the journal isn't readable.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::KernelCheckConfig;

pub struct KernelCheck {
    config: KernelCheckConfig,
}

impl KernelCheck {
    pub fn new(config: KernelCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for KernelCheck {
    fn name(&self) -> &str {
        "kernel"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Some(log) = recent_kernel_log() else {
            return Ok(unavailable());
        };
        Ok(build_result(count_incidents(&log), &self.config))
    }
}

fn build_result(incidents: usize, config: &KernelCheckConfig) -> CheckResult {
    let status_value = match incidents {
        0 => "0 incident (1 h)".to_string(),
        1 => "1 incident (1 h)".to_string(),
        n => format!("{n} incidents (1 h)"),
    };

    let mut details = vec![format!(
        "{incidents} incident(s) noyau dans la dernière heure (OOM, oops, BUG…)"
    )];
    if incidents == 0 {
        details.push("Le noyau ronronne — rien à signaler.".into());
    }

    CheckResult {
        check_name: "kernel".into(),
        metrics: vec![Metric {
            name: "kernel_incidents".into(),
            value: incidents as f64,
            unit: "events".into(),
            threshold_warning: Some(config.warning),
            threshold_critical: Some(config.critical),
        }],
        details,
        top_processes: vec![],
        status_value: Some(status_value),
    }
}

fn unavailable() -> CheckResult {
    CheckResult {
        check_name: "kernel".into(),
        metrics: vec![],
        details: vec!["Journal noyau inaccessible (groupe `systemd-journal` requis ?).".into()],
        top_processes: vec![],
        status_value: Some("Journal inaccessible".into()),
    }
}

fn recent_kernel_log() -> Option<String> {
    let output = Command::new("journalctl")
        .args([
            "-k",
            "--since",
            "1 hour ago",
            "-o",
            "cat",
            "-q",
            "--no-pager",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Count kernel-fault / OOM lines. Safe against false positives because the
/// input is the kernel ring buffer only (`journalctl -k`).
fn count_incidents(log: &str) -> usize {
    const PATTERNS: &[&str] = &[
        "out of memory",
        "oom-kill",
        "invoked oom-killer",
        "bug:",
        "oops",
        "kernel panic",
        "general protection fault",
    ];
    log.lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            PATTERNS.iter().any(|p| lower.contains(p))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> KernelCheckConfig {
        KernelCheckConfig {
            enabled: true,
            interval_secs: 300,
            warning: 1.0,
            critical: 3.0,
        }
    }

    #[test]
    fn counts_oom_and_oops_lines() {
        let log = "\
usb 1-2: new high-speed USB device
Out of memory: Killed process 1234 (chrome)
wlan0: authenticated
BUG: unable to handle kernel NULL pointer dereference
audit: type=1400
";
        assert_eq!(count_incidents(log), 2);
    }

    #[test]
    fn calm_kernel_is_info() {
        let result = build_result(0, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(result.status_value.as_deref(), Some("0 incident (1 h)"));
    }

    #[test]
    fn many_incidents_escalate() {
        let result = build_result(4, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }
}
