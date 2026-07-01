use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::check::{Check, CheckResult, Metric};
use crate::config::TemperatureThresholds;

#[derive(Debug, Clone)]
pub struct ThermalReading {
    pub label: String,
    pub celsius: f64,
}

pub struct TemperatureCheck {
    thresholds: TemperatureThresholds,
}

impl TemperatureCheck {
    pub fn new(thresholds: TemperatureThresholds) -> Self {
        Self { thresholds }
    }
}

impl Check for TemperatureCheck {
    fn name(&self) -> &str {
        "temperature"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let mut readings = read_thermal_zones()?;
        readings.extend(read_nvme_temps()?);

        if readings.is_empty() {
            return Ok(CheckResult {
                check_name: "temperature".into(),
                metrics: vec![Metric {
                    name: "temp_max_celsius".into(),
                    value: 0.0,
                    unit: "°C".into(),
                    threshold_warning: Some(self.thresholds.warning),
                    threshold_critical: Some(self.thresholds.critical),
                }],
                details: vec![
                    "Aucun capteur de température détecté.".into(),
                    "Vérifiez /sys/class/thermal ou installez lm-sensors.".into(),
                ],
                top_processes: vec![],
                status_value: Some("Aucun capteur".into()),
            });
        }

        let max = readings.iter().map(|r| r.celsius).fold(0.0_f64, f64::max);

        let hottest = readings
            .iter()
            .max_by(|a, b| a.celsius.partial_cmp(&b.celsius).unwrap())
            .map(|r| r.label.clone())
            .unwrap_or_default();

        let mut details = vec![format!("Température maximale : {max:.1} °C ({hottest})")];
        details.push("Capteurs :".into());
        for reading in &readings {
            details.push(format!("  • {} : {:.1} °C", reading.label, reading.celsius));
        }

        Ok(CheckResult {
            check_name: "temperature".into(),
            metrics: vec![Metric {
                name: "temp_max_celsius".into(),
                value: max,
                unit: "°C".into(),
                threshold_warning: Some(self.thresholds.warning),
                threshold_critical: Some(self.thresholds.critical),
            }],
            details,
            top_processes: vec![format!("Capteur le plus chaud : {hottest} ({max:.1} °C)")],
            status_value: Some(format!("{max:.0}°C")),
        })
    }
}

pub fn read_thermal_zones() -> Result<Vec<ThermalReading>> {
    let base = Path::new("/sys/class/thermal");
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut readings = Vec::new();
    for entry in fs::read_dir(base).with_context(|| format!("lecture de {}", base.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("thermal_zone") {
            continue;
        }

        let zone = entry.path();
        let temp_path = zone.join("temp");
        let type_path = zone.join("type");

        if !temp_path.exists() {
            continue;
        }

        let millis: i64 = fs::read_to_string(&temp_path)
            .with_context(|| format!("lecture de {}", temp_path.display()))?
            .trim()
            .parse()
            .with_context(|| format!("valeur invalide dans {}", temp_path.display()))?;

        if millis <= 0 {
            continue;
        }

        let label = fs::read_to_string(&type_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| name.clone());

        readings.push(ThermalReading {
            label,
            celsius: millis as f64 / 1000.0,
        });
    }

    Ok(readings)
}

pub fn read_nvme_temps() -> Result<Vec<ThermalReading>> {
    let base = Path::new("/sys/class/nvme");
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut readings = Vec::new();
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let nvme_path = entry.path();
        if !entry.file_name().to_string_lossy().starts_with("nvme") {
            continue;
        }

        let hwmon_base = nvme_path.join("hwmon");
        if !hwmon_base.exists() {
            continue;
        }

        for hwmon in fs::read_dir(&hwmon_base)? {
            let hwmon = hwmon?;
            let temp_path = hwmon.path().join("temp1_input");
            if !temp_path.exists() {
                continue;
            }

            let millis: i64 = fs::read_to_string(&temp_path)?.trim().parse()?;
            if millis <= 0 {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            readings.push(ThermalReading {
                label: format!("NVMe {name}"),
                celsius: millis as f64 / 1000.0,
            });
        }
    }

    Ok(readings)
}
