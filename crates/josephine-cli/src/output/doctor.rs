use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, ColumnConstraint, ContentArrangement, Table, Width};

use josephine_core::check::CheckResult;

use super::bars::metric_line_cell;
use super::status::state_badge;
use super::style::{
    check_label, is_tty, primary_metric, print_banner, print_footer, severity_icon,
};

const PANEL_WIDTH: u16 = 72;

pub fn print_doctor(results: &[CheckResult]) {
    print_banner("Analyse détaillée de votre système");

    for result in results {
        print_check_panel(result);
    }

    print_footer("Conseil : `josephine daemon start` pour une surveillance continue.");
    println!();
}

fn print_check_panel(result: &CheckResult) {
    let severity = result.worst_severity();
    let label = check_label(&result.check_name);
    let icon = severity_icon(severity);

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_width(PANEL_WIDTH);
    table.style_text_only();

    let title = if is_tty() {
        format!("{icon} {label}")
    } else {
        label.to_string()
    };

    table.set_header(vec![
        Cell::new(title).add_attribute(Attribute::Bold),
        state_badge(severity),
    ]);
    table.set_constraints(vec![
        ColumnConstraint::Absolute(Width::Fixed(20)),
        ColumnConstraint::LowerBoundary(Width::Fixed(40)),
    ]);

    if let Some(metric) = primary_metric(result) {
        table.add_row(vec![Cell::new("Indicateur"), metric_line_cell(metric)]);
    }

    for metric in thresholded_metrics(result) {
        table.add_row(vec![
            Cell::new(metric_label(&metric.name)),
            metric_line_cell(metric),
        ]);
    }

    for line in &result.details {
        table.add_row(vec![Cell::new(""), Cell::new(line.trim())]);
    }

    println!("{table}");
    println!();
}

fn thresholded_metrics(result: &CheckResult) -> Vec<&josephine_core::check::Metric> {
    result
        .metrics
        .iter()
        .filter(|m| {
            m.threshold_warning.is_some() && primary_metric(result).is_none_or(|p| p.name != m.name)
        })
        .collect()
}

fn metric_label(name: &str) -> String {
    match name {
        "usage_percent" => "Utilisation".into(),
        "swap_percent" => "Swap".into(),
        "usage_percent_worst" => "Disque (max)".into(),
        "temp_max_celsius" => "Température max".into(),
        "failed_units" => "Services en échec".into(),
        "max_restarts" => "Redémarrages max".into(),
        other => other.replace('_', " "),
    }
}
