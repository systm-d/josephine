use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::paths::Paths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    /// Language for all user-facing text: `en` (default) or `fr`.
    #[serde(default)]
    pub language: Lang,
    #[serde(default)]
    pub checks: ChecksConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecksConfig {
    #[serde(default)]
    pub cpu: CheckThresholds,
    #[serde(default)]
    pub memory: CheckThresholds,
    #[serde(default)]
    pub disk: CheckThresholds,
    #[serde(default)]
    pub temperature: TemperatureThresholds,
    #[serde(default)]
    pub systemd: SystemdCheckConfig,
    #[serde(default)]
    pub updates: UpdatesCheckConfig,
    #[serde(default)]
    pub network: NetworkCheckConfig,
    #[serde(default)]
    pub battery: BatteryCheckConfig,
    #[serde(default)]
    pub inode: CheckThresholds,
    #[serde(default)]
    pub smart: SmartCheckConfig,
    #[serde(default)]
    pub kernel: KernelCheckConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckThresholds {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_30")]
    pub interval_secs: u64,
    #[serde(default = "default_warning")]
    pub warning: f64,
    #[serde(default = "default_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemperatureThresholds {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_60")]
    pub interval_secs: u64,
    #[serde(default = "default_temp_warning")]
    pub warning: f64,
    #[serde(default = "default_temp_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemdCheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_120")]
    pub interval_secs: u64,
    #[serde(default = "default_failed_warning")]
    pub failed_warning: f64,
    #[serde(default = "default_failed_critical")]
    pub failed_critical: f64,
    #[serde(default = "default_restarts_warning")]
    pub restarts_warning: f64,
    #[serde(default = "default_restarts_critical")]
    pub restarts_critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatesCheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_3600")]
    pub interval_secs: u64,
    #[serde(default = "default_updates_warning")]
    pub warning: f64,
    #[serde(default = "default_updates_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkCheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_60")]
    pub interval_secs: u64,
    /// Round-trip latency to the default gateway, in milliseconds.
    #[serde(default = "default_net_warning")]
    pub warning: f64,
    #[serde(default = "default_net_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatteryCheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_120")]
    pub interval_secs: u64,
    /// Charge level (%) at or below which — while on battery — Joséphine warns.
    #[serde(default = "default_batt_warning")]
    pub warning: f64,
    #[serde(default = "default_batt_critical")]
    pub critical: f64,
}

/// SMART disk health. Off by default: `smartctl` typically needs root, so this
/// is opt-in for users who run Joséphine (or the check) with the right rights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartCheckConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval_3600")]
    pub interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelCheckConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval_300")]
    pub interval_secs: u64,
    /// Number of kernel incidents (OOM kills, oops…) in the last hour.
    #[serde(default = "default_kernel_warning")]
    pub warning: f64,
    #[serde(default = "default_kernel_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationsConfig {
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_retention")]
    pub retention_days: u32,
}

fn default_true() -> bool {
    true
}

fn default_interval_30() -> u64 {
    30
}

fn default_interval_60() -> u64 {
    60
}

fn default_interval_120() -> u64 {
    120
}

fn default_interval_3600() -> u64 {
    3600
}

fn default_updates_warning() -> f64 {
    1.0
}

fn default_updates_critical() -> f64 {
    50.0
}

fn default_net_warning() -> f64 {
    150.0
}

fn default_net_critical() -> f64 {
    500.0
}

fn default_batt_warning() -> f64 {
    20.0
}

fn default_batt_critical() -> f64 {
    10.0
}

fn default_interval_300() -> u64 {
    300
}

fn default_kernel_warning() -> f64 {
    1.0
}

fn default_kernel_critical() -> f64 {
    3.0
}

fn default_warning() -> f64 {
    85.0
}

fn default_critical() -> f64 {
    95.0
}

fn default_retention() -> u32 {
    90
}

fn default_temp_warning() -> f64 {
    75.0
}

fn default_temp_critical() -> f64 {
    90.0
}

fn default_failed_warning() -> f64 {
    1.0
}

fn default_failed_critical() -> f64 {
    3.0
}

fn default_restarts_warning() -> f64 {
    5.0
}

fn default_restarts_critical() -> f64 {
    10.0
}

impl Default for ChecksConfig {
    fn default() -> Self {
        Self {
            cpu: CheckThresholds {
                interval_secs: 30,
                ..CheckThresholds::default()
            },
            memory: CheckThresholds {
                interval_secs: 60,
                ..CheckThresholds::default()
            },
            disk: CheckThresholds {
                interval_secs: 120,
                ..CheckThresholds::default()
            },
            temperature: TemperatureThresholds::default(),
            systemd: SystemdCheckConfig::default(),
            updates: UpdatesCheckConfig::default(),
            network: NetworkCheckConfig::default(),
            battery: BatteryCheckConfig::default(),
            inode: CheckThresholds {
                interval_secs: 300,
                ..CheckThresholds::default()
            },
            smart: SmartCheckConfig::default(),
            kernel: KernelCheckConfig::default(),
        }
    }
}

impl Default for UpdatesCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: default_interval_3600(),
            warning: default_updates_warning(),
            critical: default_updates_critical(),
        }
    }
}

impl Default for NetworkCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 60,
            warning: default_net_warning(),
            critical: default_net_critical(),
        }
    }
}

impl Default for BatteryCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 120,
            warning: default_batt_warning(),
            critical: default_batt_critical(),
        }
    }
}

impl Default for SmartCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: default_interval_3600(),
        }
    }
}

impl Default for KernelCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 300,
            warning: default_kernel_warning(),
            critical: default_kernel_critical(),
        }
    }
}

impl Default for TemperatureThresholds {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 60,
            warning: default_temp_warning(),
            critical: default_temp_critical(),
        }
    }
}

impl Default for SystemdCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 120,
            failed_warning: default_failed_warning(),
            failed_critical: default_failed_critical(),
            restarts_warning: default_restarts_warning(),
            restarts_critical: default_restarts_critical(),
        }
    }
}

impl Default for CheckThresholds {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            warning: default_warning(),
            critical: default_critical(),
        }
    }
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            terminal: false,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: default_retention(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let config = if !path.exists() {
            let config = Self::default();
            config.save(path)?;
            config
        } else {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("lecture de {}", path.display()))?;
            serde_yaml::from_str(&content)
                .with_context(|| format!("analyse YAML de {}", path.display()))?
        };
        // Apply the configured language before anything user-facing is produced.
        crate::i18n::set_lang(config.language);
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        Self::validate_thresholds("cpu", &self.checks.cpu)?;
        Self::validate_thresholds("memory", &self.checks.memory)?;
        Self::validate_thresholds("disk", &self.checks.disk)?;
        Self::validate_temperature(&self.checks.temperature)?;
        Self::validate_systemd(&self.checks.systemd)?;
        Self::validate_updates(&self.checks.updates)?;
        Self::validate_network(&self.checks.network)?;
        Self::validate_battery(&self.checks.battery)?;
        Self::validate_thresholds("inode", &self.checks.inode)?;
        Self::validate_kernel(&self.checks.kernel)?;
        if self.checks.smart.interval_secs < 5 {
            bail!("checks.smart.interval_secs must be ≥ 5 seconds");
        }

        if self.history.retention_days == 0 {
            bail!("history.retention_days must be greater than 0");
        }

        Ok(())
    }

    fn validate_thresholds(name: &str, t: &CheckThresholds) -> Result<()> {
        if t.interval_secs < 5 {
            bail!("checks.{name}.interval_secs must be ≥ 5 seconds");
        }
        if !(0.0..=100.0).contains(&t.warning) {
            bail!("checks.{name}.warning must be between 0 and 100");
        }
        if !(0.0..=100.0).contains(&t.critical) {
            bail!("checks.{name}.critical must be between 0 and 100");
        }
        if t.warning >= t.critical {
            bail!("checks.{name}.warning must be less than critical");
        }
        Ok(())
    }

    fn validate_temperature(t: &TemperatureThresholds) -> Result<()> {
        if t.interval_secs < 5 {
            bail!("checks.temperature.interval_secs must be ≥ 5 seconds");
        }
        if !(20.0..=150.0).contains(&t.warning) {
            bail!("checks.temperature.warning must be between 20 and 150 °C");
        }
        if !(20.0..=150.0).contains(&t.critical) {
            bail!("checks.temperature.critical must be between 20 and 150 °C");
        }
        if t.warning >= t.critical {
            bail!("checks.temperature.warning must be less than critical");
        }
        Ok(())
    }

    fn validate_systemd(c: &SystemdCheckConfig) -> Result<()> {
        if c.interval_secs < 5 {
            bail!("checks.systemd.interval_secs must be ≥ 5 seconds");
        }
        if c.failed_warning < 1.0 || c.failed_critical < 1.0 {
            bail!("checks.systemd.failed_warning and failed_critical must be ≥ 1");
        }
        if c.failed_warning >= c.failed_critical {
            bail!("checks.systemd.failed_warning must be less than failed_critical");
        }
        if c.restarts_warning < 1.0 || c.restarts_critical < 1.0 {
            bail!("checks.systemd.restarts_warning and restarts_critical must be ≥ 1");
        }
        if c.restarts_warning >= c.restarts_critical {
            bail!("checks.systemd.restarts_warning must be less than restarts_critical");
        }
        Ok(())
    }

    fn validate_updates(c: &UpdatesCheckConfig) -> Result<()> {
        if c.interval_secs < 5 {
            bail!("checks.updates.interval_secs must be ≥ 5 seconds");
        }
        if c.warning < 1.0 || c.critical < 1.0 {
            bail!("checks.updates.warning and critical must be ≥ 1");
        }
        if c.warning >= c.critical {
            bail!("checks.updates.warning must be less than critical");
        }
        Ok(())
    }

    fn validate_network(c: &NetworkCheckConfig) -> Result<()> {
        if c.interval_secs < 5 {
            bail!("checks.network.interval_secs must be ≥ 5 seconds");
        }
        if c.warning <= 0.0 || c.critical <= 0.0 {
            bail!("checks.network.warning and critical must be positive (ms)");
        }
        if c.warning >= c.critical {
            bail!("checks.network.warning must be less than critical");
        }
        Ok(())
    }

    fn validate_battery(c: &BatteryCheckConfig) -> Result<()> {
        if c.interval_secs < 5 {
            bail!("checks.battery.interval_secs must be ≥ 5 seconds");
        }
        if !(0.0..=100.0).contains(&c.warning) || !(0.0..=100.0).contains(&c.critical) {
            bail!("checks.battery.warning and critical must be between 0 and 100 %");
        }
        // Battery thresholds are LOW-water marks: warn above critical.
        if c.critical >= c.warning {
            bail!("checks.battery.critical must be less than warning (low-water marks)");
        }
        Ok(())
    }

    fn validate_kernel(c: &KernelCheckConfig) -> Result<()> {
        if c.interval_secs < 5 {
            bail!("checks.kernel.interval_secs must be ≥ 5 seconds");
        }
        if c.warning < 1.0 || c.critical < 1.0 {
            bail!("checks.kernel.warning and critical must be ≥ 1");
        }
        if c.warning >= c.critical {
            bail!("checks.kernel.warning must be less than critical");
        }
        Ok(())
    }

    pub fn load_default() -> Result<Self> {
        let paths = Paths::new()?;
        paths.ensure_dirs()?;
        Self::load(&paths.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        Config::default().validate().unwrap();
    }

    #[test]
    fn invalid_warning_critical_rejected() {
        let mut config = Config::default();
        config.checks.cpu.warning = 96.0;
        assert!(config.validate().is_err());
    }
}
