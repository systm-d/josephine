//! SMART disk health — asks `smartctl -H` per block device and flags any drive
//! whose self-assessment isn't passing (early warning of an impending failure).
//!
//! `smartctl` usually needs root, so this check is opt-in (see config) and
//! degrades gracefully: missing tool or no read access → an informational
//! "unavailable", never a false alarm.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::SmartCheckConfig;

pub struct SmartCheck {
    #[allow(dead_code)]
    config: SmartCheckConfig,
}

impl SmartCheck {
    pub fn new(config: SmartCheckConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Health {
    Passed,
    Failing,
    Unknown,
}

impl Check for SmartCheck {
    fn name(&self) -> &str {
        "smart"
    }

    fn run(&mut self) -> Result<CheckResult> {
        if !tool_available() {
            return Ok(unavailable(
                "smartmontools non installé (paquet `smartmontools`)",
            ));
        }

        let devices = block_devices();
        let mut readable = 0;
        let mut failing = 0;
        let mut details = Vec::new();

        for device in &devices {
            match device_health(device) {
                Health::Passed => {
                    readable += 1;
                    details.push(format!("{device} : sain (SMART OK)"));
                }
                Health::Failing => {
                    readable += 1;
                    failing += 1;
                    details.push(format!("{device} : ⚠ SMART en échec — sauvegardez !"));
                }
                Health::Unknown => {}
            }
        }

        if readable == 0 {
            return Ok(unavailable(
                "état SMART illisible (droits root requis, ou disques sans SMART)",
            ));
        }

        let status_value = if failing == 0 {
            format!("{readable} disque(s) sain(s)")
        } else {
            format!("⚠ {failing} disque(s) en alerte")
        };

        Ok(CheckResult {
            check_name: "smart".into(),
            metrics: vec![Metric {
                name: "smart_failing".into(),
                value: failing as f64,
                unit: "disks".into(),
                threshold_warning: Some(1.0),
                threshold_critical: Some(1.0),
            }],
            details,
            top_processes: vec![],
            status_value: Some(status_value),
        })
    }
}

fn unavailable(reason: &str) -> CheckResult {
    CheckResult {
        check_name: "smart".into(),
        metrics: vec![],
        details: vec![reason.to_string()],
        top_processes: vec![],
        status_value: Some("Indisponible".into()),
    }
}

fn tool_available() -> bool {
    Command::new("smartctl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn block_devices() -> Vec<String> {
    let mut devices = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/block") else {
        return devices;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ["sd", "nvme", "vd", "hd"]
            .iter()
            .any(|p| name.starts_with(p))
        {
            devices.push(format!("/dev/{name}"));
        }
    }
    devices.sort();
    devices
}

fn device_health(device: &str) -> Health {
    match Command::new("smartctl").args(["-H", device]).output() {
        Ok(output) => parse_smart_health(&String::from_utf8_lossy(&output.stdout)),
        Err(_) => Health::Unknown,
    }
}

/// Read the overall-health verdict from `smartctl -H` output (ATA or NVMe).
fn parse_smart_health(stdout: &str) -> Health {
    for line in stdout.lines() {
        let line = line.to_lowercase();
        if line.contains("overall-health") || line.contains("smart health status") {
            if line.contains("passed") || line.contains("ok") {
                return Health::Passed;
            }
            if line.contains("failed") {
                return Health::Failing;
            }
        }
    }
    Health::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ata_passed() {
        let sample = "SMART overall-health self-assessment test result: PASSED\n";
        assert_eq!(parse_smart_health(sample), Health::Passed);
    }

    #[test]
    fn parses_ata_failed() {
        let sample = "SMART overall-health self-assessment test result: FAILED!\n";
        assert_eq!(parse_smart_health(sample), Health::Failing);
    }

    #[test]
    fn parses_nvme_ok() {
        let sample = "SMART Health Status: OK\n";
        assert_eq!(parse_smart_health(sample), Health::Passed);
    }

    #[test]
    fn unknown_when_absent() {
        assert_eq!(parse_smart_health("no verdict here"), Health::Unknown);
    }
}
