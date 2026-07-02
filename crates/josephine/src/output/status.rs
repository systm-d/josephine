use chrono::Local;
use colored::{Color, Colorize};
use comfy_table::{Attribute, Cell};

use josephine_core::check::{CheckResult, Severity};
use josephine_core::i18n;
use josephine_core::paths::Paths;

use super::bars::severity_color;
use super::style::{format_metric_value, is_tty, primary_metric};

/// Column (1-indexed) where every check label starts, pinned via an absolute
/// cursor move so emoji of varying display width never break alignment.
const LABEL_COLUMN: usize = 6;

const LABEL_WIDTH: usize = 20;
const VALUE_WIDTH: usize = 34;
const BOX_WIDTH: usize = 70;

pub fn print_status_table(results: &[CheckResult]) {
    print_header();

    for row in build_rows(results) {
        print_row(&row);
    }

    let global = results
        .iter()
        .map(CheckResult::worst_severity)
        .max()
        .unwrap_or(Severity::Info);

    println!();
    print_advice(global);
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
// Header: sober title block
// ---------------------------------------------------------------------------

fn print_header() {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    println!();
    // An optional user banner (e.g. large ASCII/Braille art) is stacked above
    // the title block and tinted with an amber→violet vertical gradient.
    if let Some(banner) = custom_banner() {
        print_banner_gradient(&banner);
        println!();
    }
    for line in header_lines(&ts) {
        println!("{line}");
    }
    println!("{}", "─".repeat(72).dimmed());
    println!();
}

/// Load a user banner from `<config dir>/banner.txt`, if present and non-empty.
fn custom_banner() -> Option<Vec<String>> {
    let paths = Paths::new().ok()?;
    let dir = paths.config.parent()?;
    let content = std::fs::read_to_string(dir.join("banner.txt")).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content.lines().map(str::to_string).collect())
}

/// Print each banner line tinted from amber (top) to violet (bottom).
fn print_banner_gradient(lines: &[String]) {
    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            i as f64 / (n - 1) as f64
        };
        let r = lerp(224.0, 158.0, t);
        let g = lerp(164.0, 128.0, t);
        let b = lerp(88.0, 210.0, t);
        println!("{}", line.truecolor(r, g, b));
    }
}

fn lerp(a: f64, b: f64, t: f64) -> u8 {
    (a + (b - a) * t).round() as u8
}

fn header_lines(ts: &str) -> Vec<String> {
    let heart = "♥".truecolor(240, 96, 140);
    let subtitle = i18n::t("Your system's guardian angel", "Votre ange gardien système");
    let line1 = i18n::t(
        "I watch over your machine and let you know",
        "Je veille sur votre machine et vous préviens",
    );
    let line2 = i18n::t(
        "when something's not right.",
        "quand quelque chose ne va pas.",
    );
    let last = i18n::t("Last check: ", "Dernière vérification : ");
    vec![
        format!("{}", "✨ Joséphine".truecolor(238, 108, 170).bold()),
        format!("{}", subtitle.truecolor(178, 148, 224)),
        String::new(),
        line1.to_string(),
        format!("{line2} {heart}"),
        String::new(),
        format!("{}", format!("{last}{ts}").dimmed()),
    ]
}

// ---------------------------------------------------------------------------
// Check rows
// ---------------------------------------------------------------------------

struct Row {
    icon: &'static str,
    label: &'static str,
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
    let (icon, label) = check_style(&result.check_name);
    let value = result
        .status_value
        .clone()
        .or_else(|| primary_metric(result).map(format_metric_value))
        .unwrap_or_else(|| "—".to_string());

    Row {
        icon,
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
        icon: "📈",
        label: i18n::t("System load", "Charge système"),
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

/// Emoji icon (rendered without a special font) and French label per check.
fn check_style(name: &str) -> (&'static str, &'static str) {
    match name {
        "cpu" => ("🖥️", i18n::t("CPU usage", "Utilisation CPU")),
        "memory" => ("🧠", i18n::t("Memory", "Mémoire")),
        "disk" => ("💽", i18n::t("Disk space", "Espace disque")),
        "temperature" => ("🌡️", i18n::t("Temperature", "Température")),
        "systemd" => ("🛡️", i18n::t("Critical services", "Services critiques")),
        "updates" => ("🔄", i18n::t("Updates", "Mises à jour")),
        "network" => ("🌐", i18n::t("Network", "Réseau")),
        "battery" => ("🔋", i18n::t("Battery", "Batterie")),
        "inode" => ("🗂️", "Inodes"),
        "smart" => ("💿", i18n::t("Disk health", "Santé disque")),
        "kernel" => ("🐧", i18n::t("Kernel", "Noyau")),
        _ => ("•", i18n::t("System", "Système")),
    }
}

fn print_row(row: &Row) {
    let label = pad(row.label, LABEL_WIDTH).bold();
    let value = paint(
        &pad(&row.value, VALUE_WIDTH),
        value_color(row.severity),
        false,
    );
    let badge = paint(badge_text(row.severity), value_color(row.severity), true);
    // Emoji carry their own colour, so they're printed as-is. Their display width
    // varies by terminal, so an absolute cursor move pins the label column.
    if is_tty() {
        println!("  {}\x1b[{LABEL_COLUMN}G{label}{value}{badge}", row.icon);
    } else {
        println!("  {}  {label}{value}{badge}", row.icon);
    }
}

fn value_color(severity: Severity) -> Color {
    match severity {
        Severity::Info => Color::Green,
        Severity::Attention => Color::Yellow,
        Severity::Critique => Color::Red,
    }
}

fn badge_text(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "[OK]",
        Severity::Attention => i18n::t("[!] WARNING", "[!] ATTENTION"),
        Severity::Critique => i18n::t("[✗] CRITICAL", "[✗] CRITIQUE"),
    }
}

// ---------------------------------------------------------------------------
// Advice box
// ---------------------------------------------------------------------------

fn print_advice(global: Severity) {
    let message = match global {
        Severity::Info => i18n::t(
            "All good — your machine is perfectly happy. Have a lovely day!",
            "Tout roule — votre machine file un parfait bonheur. Belle journée à vous !",
        ),
        Severity::Attention => i18n::t(
            "Nothing serious for now, but let's keep an eye on a few things.",
            "Rien de grave pour le moment, mais gardons un œil sur certaines choses.",
        ),
        Severity::Critique => i18n::t(
            "One thing deserves your attention — `josephine doctor` will tell you all.",
            "Un point mérite votre attention — `josephine doctor` vous dira tout.",
        ),
    };

    let inner = BOX_WIDTH - 2;
    let color = value_color(global);
    let tty = is_tty();

    println!("{}", format!("╭{}╮", "─".repeat(inner)).dimmed());
    for (i, line) in wrap(message, inner - 6).iter().enumerate() {
        let text = line.color(color);
        let body = if i == 0 {
            format!("  💬  {text}")
        } else {
            format!("      {text}")
        };
        if tty {
            // Pin the right border with an absolute cursor move (CHA), so the
            // varying width of 💬 can't misalign it.
            println!("{}{body}\x1b[{BOX_WIDTH}G{}", "│".dimmed(), "│".dimmed());
        } else {
            let padding = " ".repeat(inner.saturating_sub(6 + line.chars().count()));
            println!("{}{body}{padding}{}", "│".dimmed(), "│".dimmed());
        }
    }
    println!("{}", format!("╰{}╯", "─".repeat(inner)).dimmed());
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

/// Apply a colour (and optional bold) to a string via `colored`, which
/// automatically strips styling when stdout is not a terminal.
fn paint(s: &str, color: Color, bold: bool) -> String {
    let colored = s.color(color);
    if bold {
        colored.bold().to_string()
    } else {
        colored.to_string()
    }
}

/// Naive word wrap on whitespace.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
