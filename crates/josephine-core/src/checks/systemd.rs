use std::process::Command;

use anyhow::{Context, Result};

use crate::check::{Check, CheckResult, Metric};
use crate::config::SystemdCheckConfig;

pub struct SystemdCheck {
    config: SystemdCheckConfig,
}

impl SystemdCheck {
    pub fn new(config: SystemdCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for SystemdCheck {
    fn name(&self) -> &str {
        "systemd"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let snapshot = inspect_systemd()?;

        let mut details = vec![
            format!("Services en échec : {}", snapshot.failed_units.len()),
            format!(
                "Redémarrages max (service actif) : {}",
                snapshot.max_restarts
            ),
        ];

        if snapshot.failed_units.is_empty() {
            details.push("Aucun service systemd en échec.".into());
        } else {
            details.push("Services en échec :".into());
            for unit in &snapshot.failed_units {
                details.push(format!("  • {unit}"));
            }
        }

        if let Some((unit, count)) = &snapshot.max_restart_unit {
            if *count > 0 {
                details.push(format!("  • {unit} : {count} redémarrage(s)"));
            }
        }

        if !snapshot.systemd_available {
            details.push("systemctl indisponible — check systemd ignoré.".into());
        }

        Ok(CheckResult {
            check_name: "systemd".into(),
            metrics: vec![
                Metric {
                    name: "failed_units".into(),
                    value: snapshot.failed_units.len() as f64,
                    unit: "services".into(),
                    threshold_warning: Some(self.config.failed_warning),
                    threshold_critical: Some(self.config.failed_critical),
                },
                Metric {
                    name: "max_restarts".into(),
                    value: snapshot.max_restarts as f64,
                    unit: "restarts".into(),
                    threshold_warning: Some(self.config.restarts_warning),
                    threshold_critical: Some(self.config.restarts_critical),
                },
            ],
            details,
            top_processes: snapshot.failed_units.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SystemdSnapshot {
    pub failed_units: Vec<String>,
    pub max_restarts: u64,
    pub max_restart_unit: Option<(String, u64)>,
    pub systemd_available: bool,
}

pub fn inspect_systemd() -> Result<SystemdSnapshot> {
    if Command::new("systemctl")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return Ok(SystemdSnapshot {
            failed_units: vec![],
            max_restarts: 0,
            max_restart_unit: None,
            systemd_available: false,
        });
    }

    let failed_units = list_failed_units()?;
    let (max_restarts, max_restart_unit) = max_running_restarts()?;

    Ok(SystemdSnapshot {
        failed_units,
        max_restarts,
        max_restart_unit,
        systemd_available: true,
    })
}

fn list_failed_units() -> Result<Vec<String>> {
    let output = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain", "--no-pager"])
        .output()
        .context("exécution de systemctl --failed")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(parse_unit_line).collect())
}

fn max_running_restarts() -> Result<(u64, Option<(String, u64)>)> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--state=running",
            "--no-legend",
            "--plain",
            "--no-pager",
        ])
        .output()
        .context("exécution de systemctl list-units")?;

    if !output.status.success() {
        return Ok((0, None));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut max_restarts = 0_u64;
    let mut max_unit = None;

    for unit in stdout.lines().filter_map(parse_unit_line) {
        let restarts = service_restarts(&unit)?;
        if restarts > max_restarts {
            max_restarts = restarts;
            max_unit = Some((unit, restarts));
        }
    }

    Ok((max_restarts, max_unit))
}

fn service_restarts(unit: &str) -> Result<u64> {
    let output = Command::new("systemctl")
        .args(["show", unit, "-p", "NRestarts", "--value"])
        .output()
        .with_context(|| format!("lecture NRestarts pour {unit}"))?;

    if !output.status.success() {
        return Ok(0);
    }

    let value = String::from_utf8_lossy(&output.stdout);
    Ok(value.trim().parse().unwrap_or(0))
}

fn parse_unit_line(line: &str) -> Option<String> {
    let unit = line.split_whitespace().next()?.trim();
    if unit.is_empty() || unit == "0" {
        return None;
    }
    Some(unit.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unit_line_extracts_name() {
        assert_eq!(
            parse_unit_line("nginx.service loaded failed failed My Service"),
            Some("nginx.service".into())
        );
    }

    #[test]
    fn parse_unit_line_skips_empty() {
        assert_eq!(parse_unit_line(""), None);
    }
}
