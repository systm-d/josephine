//! Clock synchronisation — an unsynchronised system clock breaks log correlation,
//! TLS certificate validation and cron schedules. Reads `timedatectl show`;
//! degrades gracefully if `timedatectl` is absent.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::TimesyncCheckConfig;
use crate::i18n::{self, Lang};

pub struct TimesyncCheck {
    config: TimesyncCheckConfig,
}

impl TimesyncCheck {
    pub fn new(config: TimesyncCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for TimesyncCheck {
    fn name(&self) -> &str {
        "timesync"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Some(output) = timedatectl_show() else {
            return Ok(unavailable());
        };
        let unsynced = parse_clock_unsynced(&output);
        Ok(build_result(unsynced, &self.config))
    }
}

fn build_result(unsynced: u8, config: &TimesyncCheckConfig) -> CheckResult {
    let synced = unsynced == 0;
    let status_value = match (i18n::lang(), synced) {
        (Lang::En, true) => "in sync".to_string(),
        (Lang::En, false) => "not synchronised".to_string(),
        (Lang::Fr, true) => "synchronisée".to_string(),
        (Lang::Fr, false) => "non synchronisée".to_string(),
    };

    let mut details = vec![match (i18n::lang(), synced) {
        (Lang::En, true) => "System clock is synchronised via NTP.".to_string(),
        (Lang::En, false) => "System clock is not synchronised (NTP).".to_string(),
        (Lang::Fr, true) => "L'horloge système est synchronisée via NTP.".to_string(),
        (Lang::Fr, false) => "L'horloge système n'est pas synchronisée (NTP).".to_string(),
    }];
    if synced {
        details.push(
            i18n::t(
                "NTP is active and the clock is in sync.",
                "Le NTP est actif et l'horloge est synchronisée.",
            )
            .into(),
        );
    }

    CheckResult {
        check_name: "timesync".into(),
        metrics: vec![Metric {
            name: "clock_unsynced".into(),
            value: f64::from(unsynced),
            unit: "flag".into(),
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
        check_name: "timesync".into(),
        metrics: vec![],
        details: vec![
            i18n::t(
                "Clock status unavailable (`timedatectl` not found or failed).",
                "État de l'horloge indisponible (`timedatectl` introuvable ou en échec).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

fn timedatectl_show() -> Option<String> {
    let output = Command::new("timedatectl")
        .args(["show", "--property=NTPSynchronized", "--property=NTP"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Returns `0` when synchronised, `1` when not. Unknown/missing property → `1`
/// (conservative: surface a drift risk rather than silently OK).
fn parse_clock_unsynced(output: &str) -> u8 {
    for line in output.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("NTPSynchronized=") {
            return match value.trim() {
                "yes" => 0,
                _ => 1,
            };
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TimesyncCheckConfig {
        TimesyncCheckConfig::default()
    }

    #[test]
    fn ntp_synchronized_yes_is_synced() {
        assert_eq!(parse_clock_unsynced("NTPSynchronized=yes\nNTP=yes\n"), 0);
    }

    #[test]
    fn ntp_synchronized_no_is_unsynced() {
        assert_eq!(parse_clock_unsynced("NTPSynchronized=no\nNTP=no\n"), 1);
    }

    #[test]
    fn missing_property_is_unsynced() {
        assert_eq!(parse_clock_unsynced("NTP=no\n"), 1);
    }

    #[test]
    fn synced_is_info() {
        let result = build_result(0, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(result.status_value.as_deref(), Some("in sync"));
    }

    #[test]
    fn unsynced_is_attention_not_critical() {
        let result = build_result(1, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Attention);
        assert_eq!(result.status_value.as_deref(), Some("not synchronised"));
    }
}
