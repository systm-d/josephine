use colored::Colorize;
use comfy_table::{Attribute, Cell};

use josephine_core::check::{CheckResult, Severity};
use josephine_core::i18n;

use super::bars::severity_color;
use super::style::{format_metric_value, primary_metric};

const LABEL_WIDTH: usize = 14;

pub fn print_status_table(results: &[CheckResult]) {
    super::style::sober_header(
        None,
        Some(i18n::t(
            "Your machine, watched over.",
            "Votre machine, sous bonne garde.",
        )),
    );
    for row in build_rows(results) {
        print_row(&row);
    }
    println!("{}", "─".repeat(super::style::HEADER_WIDTH).dimmed());
    print_footer_line(results);
}

/// Badge cell used by the `doctor` detailed table (kept for that view).
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

// ---------------------------------------------------------------------------
// Check rows
// ---------------------------------------------------------------------------

struct Row {
    label: String,
    value: String,
    severity: Severity,
}

fn build_rows(results: &[CheckResult]) -> Vec<Row> {
    let mut rows = Vec::new();
    for result in results {
        rows.push(check_row(result));
        // The system load lives on the CPU check; surface it as its own line.
        if result.check_name == "cpu"
            && let Some(load_row) = load_row()
        {
            rows.push(load_row);
        }
    }
    rows
}

fn check_row(result: &CheckResult) -> Row {
    let label = super::style::check_label(&result.check_name).to_string();
    let value = result
        .status_value
        .clone()
        .or_else(|| primary_metric(result).map(format_metric_value))
        .unwrap_or_else(|| "—".to_string());

    Row {
        label,
        value,
        severity: result.worst_severity(),
    }
}

fn load_row() -> Option<Row> {
    let (one, five, fifteen) = read_loadavg()?;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as f64;
    let ratio = one / cores.max(1.0);
    let severity = if ratio >= 2.0 {
        Severity::Critique
    } else if ratio >= 1.0 {
        Severity::Attention
    } else {
        Severity::Info
    };

    Some(Row {
        label: i18n::t("Load", "Charge").to_string(),
        value: format!("{one:.2} (1m) {five:.2} (5m) {fifteen:.2} (15m)"),
        severity,
    })
}

fn read_loadavg() -> Option<(f64, f64, f64)> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut it = content.split_whitespace();
    let one = it.next()?.parse().ok()?;
    let five = it.next()?.parse().ok()?;
    let fifteen = it.next()?.parse().ok()?;
    Some((one, five, fifteen))
}

fn print_row(row: &Row) {
    let glyph = super::style::status_glyph(row.severity);
    let label = pad(&row.label, LABEL_WIDTH);
    let value = super::style::severity_paint(&row.value, row.severity);
    println!(" {glyph}  {label}{value}");
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

fn print_footer_line(results: &[CheckResult]) {
    let n = results
        .iter()
        .filter(|r| r.worst_severity() != Severity::Info)
        .count();
    let msg = if n == 0 {
        i18n::t("All clear.", "Tout est au vert.").to_string()
    } else {
        match i18n::lang() {
            josephine_core::i18n::Lang::En => format!(
                "{n} thing{} to look at → josephine doctor",
                if n > 1 { "s" } else { "" }
            ),
            josephine_core::i18n::Lang::Fr => format!(
                "{n} point{} à regarder → josephine doctor",
                if n > 1 { "s" } else { "" }
            ),
        }
    };
    println!(
        " {}",
        if super::style::is_tty() {
            msg.dimmed().to_string()
        } else {
            msg
        }
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pad a string to `width` display columns (approximated by char count).
fn pad(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - len))
    }
}
