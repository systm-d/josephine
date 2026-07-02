use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use colored::Colorize;
use josephine_core::check::{Metric, metric_severity};
use josephine_core::i18n::{self, Lang};

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Ask a yes/no question on the terminal. Returns `false` on a non-interactive
/// stdin (so nothing destructive ever runs unattended without `--yes`).
pub fn confirm(question: &str) -> Result<bool> {
    if !is_tty() {
        return Ok(false);
    }
    print!("{question}{}", i18n::t(" [y/N] ", " [o/N] "));
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_lowercase().as_str(),
        "o" | "oui" | "y" | "yes"
    ))
}

/// Render a compact unicode sparkline (`▁▂▃▅▇`) from a series of values,
/// scaled between the series' own min and max.
pub fn sparkline(values: &[f64]) -> String {
    const TICKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return "—".to_string();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    values
        .iter()
        .map(|&v| {
            let idx = if span <= f64::EPSILON {
                TICKS.len() / 2
            } else {
                (((v - min) / span) * (TICKS.len() - 1) as f64).round() as usize
            };
            TICKS[idx.min(TICKS.len() - 1)]
        })
        .collect()
}

pub fn check_label(name: &str) -> &'static str {
    match name {
        "cpu" => "CPU",
        "memory" => i18n::t("Memory", "Mémoire"),
        "disk" => i18n::t("Disk", "Disque"),
        "temperature" => i18n::t("Temperature", "Température"),
        "systemd" => i18n::t("systemd services", "Services systemd"),
        "updates" => i18n::t("Updates", "Mises à jour"),
        "network" => i18n::t("Network", "Réseau"),
        "battery" => i18n::t("Battery", "Batterie"),
        "inode" => "Inodes",
        "smart" => i18n::t("Disk health", "Santé disque"),
        "kernel" => i18n::t("Kernel", "Noyau"),
        _ => i18n::t("System", "Système"),
    }
}

pub fn print_banner(subtitle: &str) {
    if is_tty() {
        println!();
        println!("{}", "✨ Joséphine".to_string().bold().cyan());
        println!("{}", subtitle.dimmed());
        println!("{}", "─".repeat(52).dimmed());
        println!();
    } else {
        println!("✨ Joséphine");
        println!("{subtitle}");
        println!();
    }
}

pub fn print_footer(message: &str) {
    if is_tty() {
        println!("{}", message.dimmed());
    } else {
        println!("{message}");
    }
}

pub fn primary_metric(result: &josephine_core::check::CheckResult) -> Option<&Metric> {
    match result.check_name.as_str() {
        "cpu" => result.metrics.iter().find(|m| m.name == "usage_percent"),
        "memory" => result.metrics.iter().find(|m| m.name == "usage_percent"),
        "disk" => result
            .metrics
            .iter()
            .find(|m| m.name == "usage_percent_worst"),
        "temperature" => result.metrics.iter().find(|m| m.name == "temp_max_celsius"),
        "systemd" => result
            .metrics
            .iter()
            .max_by_key(|m| (metric_severity(m), m.name == "failed_units")),
        "network" => result
            .metrics
            .iter()
            .find(|m| m.name == "gateway_latency_ms"),
        "battery" => result.metrics.iter().find(|m| m.name == "charge_percent"),
        "inode" => result
            .metrics
            .iter()
            .find(|m| m.name == "inode_usage_percent_worst"),
        "smart" => result.metrics.iter().find(|m| m.name == "smart_failing"),
        "kernel" => result.metrics.iter().find(|m| m.name == "kernel_incidents"),
        _ => result.metrics.first(),
    }
}

pub fn metric_scale(metric: &Metric) -> f64 {
    metric
        .threshold_critical
        .or(metric.threshold_warning)
        .unwrap_or(100.0)
        .max(1.0)
}

pub fn format_metric_value(metric: &Metric) -> String {
    match metric.unit.as_str() {
        "%" => format!("{:.1} %", metric.value),
        "°C" => format!("{:.1} °C", metric.value),
        "services" => {
            let n = metric.value as u64;
            if n <= 1 {
                format!("{n} service")
            } else {
                format!("{n} services")
            }
        }
        "restarts" => {
            let n = metric.value as u64;
            match (i18n::lang(), n) {
                (Lang::En, 0..=1) => format!("{n} restart"),
                (Lang::En, _) => format!("{n} restarts"),
                (Lang::Fr, 0..=1) => format!("{n} redémarrage"),
                (Lang::Fr, _) => format!("{n} redémarrages"),
            }
        }
        "updates" => {
            let n = metric.value as u64;
            match (i18n::lang(), n) {
                (Lang::En, 0) => "up to date".to_string(),
                (Lang::En, 1) => "1 update".to_string(),
                (Lang::En, _) => format!("{n} updates"),
                (Lang::Fr, 0) => "à jour".to_string(),
                (Lang::Fr, 1) => "1 mise à jour".to_string(),
                (Lang::Fr, _) => format!("{n} mises à jour"),
            }
        }
        "ms" => format!("{:.0} ms", metric.value),
        "disks" => {
            let n = metric.value as u64;
            match i18n::lang() {
                Lang::En => format!("{n} disk(s)"),
                Lang::Fr => format!("{n} disque(s)"),
            }
        }
        "events" => {
            let n = metric.value as u64;
            format!("{n} incident(s)")
        }
        _ => format!("{:.1} {}", metric.value, metric.unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_empty_is_dash() {
        assert_eq!(sparkline(&[]), "—");
    }

    #[test]
    fn sparkline_flat_series_is_uniform() {
        let line = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(line.chars().count(), 3);
        // A flat series maps every point to the middle tick.
        assert!(line.chars().all(|c| c == '▅'));
    }

    #[test]
    fn sparkline_rises_with_values() {
        let line = sparkline(&[0.0, 50.0, 100.0]);
        let ticks: Vec<char> = line.chars().collect();
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0], '▁');
        assert_eq!(ticks[2], '█');
        assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2]);
    }
}
