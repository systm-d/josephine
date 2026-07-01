//! Battery check — reads `/sys/class/power_supply`. Unlike the other checks,
//! *low* is bad, so severity is expressed via an inverted "depletion" metric
//! that only appears when actually discharging and low (no false alarms while
//! plugged in). Gracefully reports "no battery" on desktops.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::BatteryCheckConfig;

pub struct BatteryCheck {
    config: BatteryCheckConfig,
}

impl BatteryCheck {
    pub fn new(config: BatteryCheckConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone)]
struct BatteryReading {
    name: String,
    capacity: f64,
    status: String,
    /// Current full charge as a % of the design capacity, if exposed.
    health: Option<f64>,
}

impl Check for BatteryCheck {
    fn name(&self) -> &str {
        "battery"
    }

    fn run(&mut self) -> Result<CheckResult> {
        Ok(build_result(&read_batteries(), &self.config))
    }
}

/// Pure result builder — keeps the severity logic unit-testable.
fn build_result(batteries: &[BatteryReading], config: &BatteryCheckConfig) -> CheckResult {
    let Some(batt) = batteries
        .iter()
        .min_by(|a, b| a.capacity.total_cmp(&b.capacity))
    else {
        return CheckResult {
            check_name: "battery".into(),
            metrics: vec![],
            details: vec!["Aucune batterie détectée (poste fixe ?).".into()],
            top_processes: vec![],
            status_value: Some("Pas de batterie".into()),
        };
    };

    let discharging = batt.status.eq_ignore_ascii_case("Discharging");
    let state = translate_status(&batt.status);

    // Charge level: informational (drives the display bar), never alarming.
    // Depletion (100 − charge, but only while discharging) carries the severity
    // and is ALWAYS present so the rules engine can fire a recovery once the
    // machine is plugged back in.
    let depletion = if discharging {
        100.0 - batt.capacity
    } else {
        0.0
    };
    let metrics = vec![
        Metric {
            name: "charge_percent".into(),
            value: batt.capacity,
            unit: "%".into(),
            threshold_warning: None,
            threshold_critical: None,
        },
        Metric {
            name: "battery_depletion_percent".into(),
            value: depletion,
            unit: "%".into(),
            threshold_warning: Some(100.0 - config.warning),
            threshold_critical: Some(100.0 - config.critical),
        },
    ];

    let mut details = vec![format!(
        "{} : {:.0} % — {}",
        batt.name, batt.capacity, state
    )];
    if let Some(health) = batt.health {
        details.push(format!("Santé : {health:.0} % de la capacité d'origine"));
    }

    CheckResult {
        check_name: "battery".into(),
        metrics,
        details,
        top_processes: vec![],
        status_value: Some(format!("{:.0} % ({state})", batt.capacity)),
    }
}

fn translate_status(status: &str) -> &'static str {
    match status.trim() {
        "Charging" => "en charge",
        "Discharging" => "sur batterie",
        "Full" => "pleine",
        "Not charging" => "branchée",
        _ => "état inconnu",
    }
}

fn read_batteries() -> Vec<BatteryReading> {
    let base = Path::new("/sys/class/power_supply");
    let mut batteries = Vec::new();
    let Ok(entries) = fs::read_dir(base) else {
        return batteries;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if read_trim(&path.join("type")).as_deref() != Some("Battery") {
            continue;
        }
        let Some(capacity) = read_trim(&path.join("capacity")).and_then(|s| s.parse::<f64>().ok())
        else {
            continue;
        };
        batteries.push(BatteryReading {
            name: entry.file_name().to_string_lossy().to_string(),
            capacity,
            status: read_trim(&path.join("status")).unwrap_or_else(|| "Unknown".into()),
            health: read_health(&path),
        });
    }
    batteries
}

fn read_health(path: &Path) -> Option<f64> {
    let (full, design) = if path.join("energy_full").exists() {
        (
            read_num(&path.join("energy_full"))?,
            read_num(&path.join("energy_full_design"))?,
        )
    } else {
        (
            read_num(&path.join("charge_full"))?,
            read_num(&path.join("charge_full_design"))?,
        )
    };
    (design > 0.0).then(|| (full / design * 100.0).min(100.0))
}

fn read_trim(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_num(path: &Path) -> Option<f64> {
    read_trim(path).and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(capacity: f64, status: &str) -> BatteryReading {
        BatteryReading {
            name: "BAT0".into(),
            capacity,
            status: status.into(),
            health: None,
        }
    }

    fn config() -> BatteryCheckConfig {
        BatteryCheckConfig {
            enabled: true,
            interval_secs: 120,
            warning: 20.0,
            critical: 10.0,
        }
    }

    #[test]
    fn no_battery_is_informational() {
        let result = build_result(&[], &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(result.status_value.as_deref(), Some("Pas de batterie"));
    }

    #[test]
    fn low_but_charging_does_not_alarm() {
        let result = build_result(&[reading(8.0, "Charging")], &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
    }

    #[test]
    fn discharging_low_warns() {
        let result = build_result(&[reading(15.0, "Discharging")], &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Attention);
    }

    #[test]
    fn discharging_very_low_is_critical() {
        let result = build_result(&[reading(5.0, "Discharging")], &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }

    #[test]
    fn discharging_healthy_level_is_calm() {
        let result = build_result(&[reading(80.0, "Discharging")], &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
    }
}
