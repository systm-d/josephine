//! Security signals — repeated failed logins and sudo attempts in the last hour
//! are worth surfacing early. Reads `journalctl`; degrades gracefully if the
//! journal is unreadable.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::SecurityCheckConfig;
use crate::i18n::{self, Lang};

pub struct SecurityCheck {
    config: SecurityCheckConfig,
}

impl SecurityCheck {
    pub fn new(config: SecurityCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for SecurityCheck {
    fn name(&self) -> &str {
        "security"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Some(log) = recent_auth_log() else {
            return Ok(unavailable());
        };
        Ok(build_result(count_failed_auths(&log), &self.config))
    }
}

fn build_result(count: usize, config: &SecurityCheckConfig) -> CheckResult {
    let status_value = match (i18n::lang(), count) {
        (Lang::En, 0) => "no failed logins (1 h)".to_string(),
        (Lang::En, 1) => "1 failed login (1 h)".to_string(),
        (Lang::En, n) => format!("{n} failed logins (1 h)"),
        (Lang::Fr, 0) => "aucune connexion échouée (1 h)".to_string(),
        (Lang::Fr, 1) => "1 connexion échouée (1 h)".to_string(),
        (Lang::Fr, n) => format!("{n} connexions échouées (1 h)"),
    };

    let mut details = vec![match (i18n::lang(), count) {
        (Lang::En, 0) => "No failed authentication attempts in the last hour.".to_string(),
        (Lang::En, n) => format!("{n} failed authentication attempt(s) in the last hour."),
        (Lang::Fr, 0) => {
            "Aucune tentative d'authentification échouée sur la dernière heure.".to_string()
        }
        (Lang::Fr, n) => {
            format!("{n} tentative(s) d'authentification échouée(s) sur la dernière heure.")
        }
    }];
    if count == 0 {
        details.push(
            i18n::t(
                "No suspicious login activity in the last hour.",
                "Aucune activité de connexion suspecte sur la dernière heure.",
            )
            .into(),
        );
    }

    CheckResult {
        check_name: "security".into(),
        metrics: vec![Metric {
            name: "failed_auths".into(),
            value: count as f64,
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
        check_name: "security".into(),
        metrics: vec![],
        details: vec![
            i18n::t(
                "Authentication journal unreadable (systemd-journal group required?).",
                "Journal d'authentification inaccessible (groupe `systemd-journal` requis ?).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

fn recent_auth_log() -> Option<String> {
    let output = Command::new("journalctl")
        .args(["--since", "1 hour ago", "-o", "cat", "-q", "--no-pager"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Count lines matching common failed-auth patterns (case-insensitive).
fn count_failed_auths(log: &str) -> usize {
    const PATTERNS: &[&str] = &["failed password", "authentication failure", "invalid user"];
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

    fn config() -> SecurityCheckConfig {
        SecurityCheckConfig::default()
    }

    const SAMPLE_LOG: &str = "\
Accepted publickey for alice from 10.0.0.1 port 22
Failed password for invalid user admin from 203.0.113.5 port 45678 ssh2
Failed password for root from 203.0.113.5 port 45679 ssh2
pam_unix(sshd:auth): authentication failure; logname= uid=0
Invalid user guest from 203.0.113.6 port 12345
session opened for user alice
";

    #[test]
    fn counts_failed_auth_lines() {
        assert_eq!(count_failed_auths(SAMPLE_LOG), 4);
    }

    #[test]
    fn calm_journal_is_info() {
        let result = build_result(0, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(
            result.status_value.as_deref(),
            Some("no failed logins (1 h)")
        );
    }

    #[test]
    fn many_failures_escalate_to_critical() {
        let result = build_result(20, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }

    #[test]
    fn moderate_failures_are_attention() {
        let result = build_result(7, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Attention);
    }
}
