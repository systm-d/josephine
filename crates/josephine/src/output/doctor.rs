use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ColumnConstraint, ContentArrangement, Table, Width};

use josephine_core::check::{CheckResult, Metric, Severity};
use josephine_core::checks::interval_for_check;
use josephine_core::config::Config;

use super::bars::{BAR_WIDTH, bar_plain, severity_color};
use super::status::state_badge;
use super::style::{
    check_label, format_metric_value, metric_scale, primary_metric, print_banner, print_footer,
};

const TABLE_WIDTH: u16 = 86;

pub fn print_doctor(results: &[CheckResult], config: &Config, verbose: bool) {
    print_banner("Analyse détaillée de votre système");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_width(TABLE_WIDTH);
    table.set_header(vec![
        Cell::new("Check").add_attribute(Attribute::Bold),
        Cell::new("État").add_attribute(Attribute::Bold),
        Cell::new("Mesure & détails").add_attribute(Attribute::Bold),
    ]);
    table.set_constraints(vec![
        ColumnConstraint::Absolute(Width::Fixed(13)),
        ColumnConstraint::Absolute(Width::Fixed(7)),
        ColumnConstraint::LowerBoundary(Width::Fixed(48)),
    ]);

    for result in results {
        let severity = result.worst_severity();
        let mut detail_cell = Cell::new(detail_lines(result, config, verbose).join("\n"));
        if severity != Severity::Info {
            detail_cell = detail_cell.fg(severity_color(severity));
        }
        table.add_row(vec![
            Cell::new(check_label(&result.check_name)),
            state_badge(severity),
            detail_cell,
        ]);
    }

    println!("{table}");

    if verbose {
        print_footer("Vue condensée : `josephine doctor` (sans --verbose).");
    } else {
        print_footer("Tout voir (seuils, processus, intervalles) : `josephine doctor --verbose`.");
    }
    println!();
}

/// Compose the multi-line "Mesure & détails" cell for one check.
fn detail_lines(result: &CheckResult, config: &Config, verbose: bool) -> Vec<String> {
    let mut lines = Vec::new();

    // Thresholded metrics as bar lines — primary first, then the rest (swap, …).
    let primary = primary_metric(result);
    let primary_name = primary.map(|m| m.name.as_str());
    let mut ordered: Vec<&Metric> = Vec::new();
    if let Some(p) = primary {
        ordered.push(p);
    }
    for metric in &result.metrics {
        if metric.threshold_warning.is_some() && Some(metric.name.as_str()) != primary_name {
            ordered.push(metric);
        }
    }

    for (i, metric) in ordered.iter().enumerate() {
        let bar = bar_plain(metric.value, metric_scale(metric), BAR_WIDTH);
        let value = format_metric_value(metric);
        if i == 0 {
            lines.push(format!("{bar}  {value}"));
        } else {
            lines.push(format!("{}  {bar}  {value}", metric_label(&metric.name)));
        }
        if verbose && let (Some(w), Some(c)) = (metric.threshold_warning, metric.threshold_critical)
        {
            lines.push(format!(
                "  seuil alerte {} · critique {}",
                fmt_threshold(w, &metric.unit),
                fmt_threshold(c, &metric.unit)
            ));
        }
    }

    // System facts (load, sensors, partitions, failed services…).
    for detail in &result.details {
        lines.push(detail.trim().to_string());
    }

    // Process lists live in `top_processes` for cpu/memory — 3 normally, 10 in verbose.
    if let Some(header) = process_header(&result.check_name) {
        let limit = if verbose { 10 } else { 3 };
        let processes: Vec<&String> = result.top_processes.iter().take(limit).collect();
        if !processes.is_empty() {
            lines.push(header.to_string());
            for process in processes {
                lines.push(format!("• {process}"));
            }
        }
    }

    if verbose {
        let secs = interval_for_check(&result.check_name, &config.checks);
        lines.push(format!("Collecte {}", human_interval(secs)));
    }

    lines
}

fn process_header(check_name: &str) -> Option<&'static str> {
    match check_name {
        "cpu" => Some("Processus les plus actifs :"),
        "memory" => Some("Processus les plus gourmands :"),
        _ => None,
    }
}

fn fmt_threshold(value: f64, unit: &str) -> String {
    match unit {
        "%" => format!("{value:.0} %"),
        "°C" => format!("{value:.0} °C"),
        "ms" => format!("{value:.0} ms"),
        _ => format!("{value:.0}"),
    }
}

fn human_interval(secs: u64) -> String {
    if secs >= 3600 && secs % 3600 == 0 {
        format!("toutes les {} h", secs / 3600)
    } else if secs >= 60 && secs % 60 == 0 {
        format!("toutes les {} min", secs / 60)
    } else {
        format!("toutes les {secs} s")
    }
}

fn metric_label(name: &str) -> String {
    match name {
        "usage_percent" => "Utilisation".into(),
        "swap_percent" => "Swap".into(),
        "usage_percent_worst" => "Disque (max)".into(),
        "temp_max_celsius" => "Température max".into(),
        "failed_units" => "Services en échec".into(),
        "max_restarts" => "Redémarrages max".into(),
        "updates_available" => "Mises à jour".into(),
        "gateway_latency_ms" => "Latence passerelle".into(),
        "charge_percent" => "Charge".into(),
        "battery_depletion_percent" => "Décharge".into(),
        other => other.replace('_', " "),
    }
}
