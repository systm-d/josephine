use comfy_table::Color;
use josephine_core::check::{metric_severity, Metric, Severity};

pub const BAR_WIDTH: usize = 16;

pub fn bar_plain(value: f64, scale: f64, width: usize) -> String {
    let ratio = (value / scale.max(1.0)).clamp(0.0, 1.0);
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Info => Color::Green,
        Severity::Attention => Color::Yellow,
        Severity::Critique => Color::Red,
    }
}

pub fn metric_measure_cell(metric: &Metric) -> comfy_table::Cell {
    use comfy_table::Cell;

    let severity = metric_severity(metric);
    let scale = super::style::metric_scale(metric);
    let bar = bar_plain(metric.value, scale, BAR_WIDTH);
    let value = super::style::format_metric_value(metric);
    Cell::new(format!("{bar}  {value:>10}")).fg(severity_color(severity))
}

pub fn metric_line_cell(metric: &Metric) -> comfy_table::Cell {
    metric_measure_cell(metric)
}
