use colored::Colorize;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{
    Attribute, Cell, ColumnConstraint, ContentArrangement, Table, Width,
};

use josephine_core::check::{CheckResult, Severity};

use super::bars::{metric_measure_cell, severity_color, BAR_WIDTH};
use super::style::{
    check_label, is_tty, primary_metric, print_banner, severity_icon,
};

const TABLE_WIDTH: u16 = 72;

pub fn print_status_table(results: &[CheckResult]) {
    print_banner("Résumé de l'état de votre machine");

    if is_tty() {
        print_status_table_rich(results);
    } else {
        print_status_table_plain(results);
    }

    let global = results
        .iter()
        .map(CheckResult::worst_severity)
        .max()
        .unwrap_or(Severity::Info);

    println!();
    print_global_summary(global);
}

pub fn state_badge(severity: Severity) -> Cell {
    let label = match severity {
        Severity::Info => " ok ",
        Severity::Attention => "alert",
        Severity::Critique => "crit",
    };
    Cell::new(label)
        .fg(severity_color(severity))
        .add_attribute(Attribute::Bold)
}

fn print_status_table_rich(results: &[CheckResult]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_width(TABLE_WIDTH);
    table.style_text_only();
    table.set_header(vec![
        Cell::new("Check").add_attribute(Attribute::Bold),
        Cell::new("Mesure").add_attribute(Attribute::Bold),
        Cell::new("État").add_attribute(Attribute::Bold),
    ]);
    table.set_constraints(vec![
        ColumnConstraint::Absolute(Width::Fixed(16)),
        ColumnConstraint::LowerBoundary(Width::Fixed(36)),
        ColumnConstraint::Absolute(Width::Fixed(8)),
    ]);

    for result in results {
        table.add_row(status_row(result));
    }

    println!("{table}");
}

fn print_status_table_plain(results: &[CheckResult]) {
    println!(
        "{:<16}  {:<16}  {:>10}  {:>8}",
        "CHECK", "BARRE", "VALEUR", "ÉTAT",
    );
    println!("{}", "─".repeat(TABLE_WIDTH as usize));

    for result in results {
        let label = status_check_label(&result.check_name);
        let severity = result.worst_severity();
        let state = plain_state(severity);

        if let Some(metric) = primary_metric(result) {
            let scale = super::style::metric_scale(metric);
            let bar = super::bars::bar_plain(metric.value, scale, BAR_WIDTH);
            let value = super::style::format_metric_value(metric);
            println!("{label:<16}  {bar}  {value:>10}  {state:>8}");
        } else {
            println!("{label:<16}  {dash:<28}  {state:>8}", dash = "—");
        }
    }
}

fn status_row(result: &CheckResult) -> Vec<Cell> {
    let severity = result.worst_severity();
    let label = status_check_label(&result.check_name);
    let icon = severity_icon(severity);

    let check_cell = Cell::new(format!("{icon} {label}")).add_attribute(Attribute::Bold);

    let measure_cell = if result.check_name == "systemd" {
        systemd_measure_cell(result)
    } else {
        primary_metric(result)
            .map(metric_measure_cell)
            .unwrap_or_else(|| Cell::new("—"))
    };

    let state_cell = state_badge(severity);

    vec![check_cell, measure_cell, state_cell]
}

fn systemd_measure_cell(result: &CheckResult) -> Cell {
    use comfy_table::Color;

    let failed = result
        .metrics
        .iter()
        .find(|m| m.name == "failed_units");
    let restarts = result
        .metrics
        .iter()
        .find(|m| m.name == "max_restarts");

    let failed_count = failed.map(|m| m.value as u64).unwrap_or(0);
    let restart_count = restarts.map(|m| m.value as u64).unwrap_or(0);

    if failed_count == 0 && restart_count == 0 {
        return Cell::new("Aucun souci détecté").fg(Color::Green);
    }

    primary_metric(result)
        .map(metric_measure_cell)
        .unwrap_or_else(|| Cell::new("—"))
}

fn status_check_label(name: &str) -> String {
    match name {
        "systemd" => "Systemd".into(),
        other => check_label(other).to_string(),
    }
}

fn plain_state(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "ok",
        Severity::Attention => "attention",
        Severity::Critique => "critique",
    }
}

fn print_global_summary(global: Severity) {
    let message = match global {
        Severity::Info => "Votre machine va bien.",
        Severity::Attention => {
            "Joséphine a remarqué quelque chose — consultez `josephine doctor`."
        }
        Severity::Critique => "Une intervention serait utile — lancez `josephine doctor`.",
    };

    if !is_tty() {
        println!("✨ {message}");
        return;
    }

    match global {
        Severity::Info => println!("{}", format!("✨ {message}").green()),
        Severity::Attention => println!("{}", format!("✨ {message}").yellow()),
        Severity::Critique => println!("{}", format!("✨ {message}").red()),
    }
}
