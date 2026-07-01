use std::io::IsTerminal;

use colored::Colorize;
use josephine_core::check::{Metric, metric_severity};

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

pub fn check_label(name: &str) -> &'static str {
    match name {
        "cpu" => "CPU",
        "memory" => "Mémoire",
        "disk" => "Disque",
        "temperature" => "Température",
        "systemd" => "Services systemd",
        "updates" => "Mises à jour",
        _ => "Système",
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
            if n <= 1 {
                format!("{n} redémarrage")
            } else {
                format!("{n} redémarrages")
            }
        }
        "updates" => {
            let n = metric.value as u64;
            match n {
                0 => "à jour".to_string(),
                1 => "1 mise à jour".to_string(),
                _ => format!("{n} mises à jour"),
            }
        }
        _ => format!("{:.1} {}", metric.value, metric.unit),
    }
}
