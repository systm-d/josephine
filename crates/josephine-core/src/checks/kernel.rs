//! Kernel incidents — scans the recent kernel journal for out-of-memory kills
//! and faults (oops / BUG / panic): the quiet events that destabilise a machine.
//! Reads `journalctl -k`; degrades gracefully if the journal isn't readable.

use std::sync::Arc;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::KernelCheckConfig;
use crate::i18n::{self, Lang};
use crate::source::{Commands, system_commands};

pub struct KernelCheck {
    config: KernelCheckConfig,
    commands: Arc<dyn Commands>,
}

impl KernelCheck {
    pub fn new(config: KernelCheckConfig) -> Self {
        Self {
            config,
            commands: system_commands(),
        }
    }

    /// Read the journal from a stub instead of running `journalctl`.
    pub fn with_commands(mut self, commands: Arc<dyn Commands>) -> Self {
        self.commands = commands;
        self
    }
}

impl Check for KernelCheck {
    fn name(&self) -> &str {
        "kernel"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Some(log) = recent_kernel_log(self.commands.as_ref()) else {
            return Ok(unavailable());
        };
        Ok(build_result(count_incidents(&log), &self.config))
    }
}

fn build_result(incidents: usize, config: &KernelCheckConfig) -> CheckResult {
    let status_value = match (i18n::lang(), incidents) {
        (Lang::En, 1) => "1 incident (1 h)".to_string(),
        (Lang::En, n) => format!("{n} incidents (1 h)"),
        (Lang::Fr, 1) => "1 incident (1 h)".to_string(),
        (Lang::Fr, n) => format!("{n} incidents (1 h)"),
    };

    let mut details = vec![match i18n::lang() {
        Lang::En => {
            format!("{incidents} kernel incident(s) in the last hour (OOM, oops, BUG…)")
        }
        Lang::Fr => {
            format!("{incidents} incident(s) noyau dans la dernière heure (OOM, oops, BUG…)")
        }
    }];
    if incidents == 0 {
        details.push(
            i18n::t(
                "No kernel incidents in the last hour.",
                "Aucun incident noyau sur la dernière heure.",
            )
            .into(),
        );
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
        details: vec![
            i18n::t(
                "Kernel journal unreadable (systemd-journal group required?).",
                "Journal noyau inaccessible (groupe `systemd-journal` requis ?).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Journal unreadable", "Journal inaccessible").into()),
    }
}

fn recent_kernel_log(commands: &dyn Commands) -> Option<String> {
    commands.output(
        "journalctl",
        &[
            "-k",
            "--since",
            "1 hour ago",
            "-o",
            "cat",
            "-q",
            "--no-pager",
        ],
    )
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
        assert_eq!(result.status_value.as_deref(), Some("0 incidents (1 h)"));
    }

    #[test]
    fn many_incidents_escalate() {
        let result = build_result(4, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }
}
