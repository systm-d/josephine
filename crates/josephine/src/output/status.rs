use chrono::Local;
use colored::{Color, Colorize};
use comfy_table::{Attribute, Cell};

use josephine_core::check::{CheckResult, Severity};
use josephine_core::paths::Paths;

use super::bars::severity_color;
use super::style::{format_metric_value, is_tty, primary_metric};

/// Column (1-indexed) where every check label starts, pinned via an absolute
/// cursor move so emoji of varying display width never break alignment.
const LABEL_COLUMN: usize = 6;

/// Fixed width (in columns) reserved for the angel art on the left.
const ART_WIDTH: usize = 24;
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
// Header: angel art beside the title block
// ---------------------------------------------------------------------------

fn print_header() {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let header = header_lines(&ts);

    println!();
    match custom_banner() {
        // A user-provided banner (e.g. large ASCII/Braille art) is stacked above
        // the title block and tinted with an amber→violet vertical gradient.
        Some(banner) => {
            print_banner_gradient(&banner);
            println!();
            for line in &header {
                println!("{line}");
            }
        }
        // Default: the built-in angel sits beside the title block.
        None => {
            let art = angel_art();
            let rows = art.len().max(header.len());
            for i in 0..rows {
                let left = match art.get(i) {
                    Some((text, color)) => render_art_line(text, *color),
                    None => " ".repeat(ART_WIDTH),
                };
                let right = header.get(i).cloned().unwrap_or_default();
                println!("{left}  {right}");
            }
        }
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

/// The angel: halo, face, spread wings, robe and a heart. Each line carries its
/// own colour; a `♥` anywhere is always painted pink (see `render_art_line`).
fn angel_art() -> Vec<(&'static str, Color)> {
    let amber = Color::TrueColor {
        r: 224,
        g: 164,
        b: 88,
    };
    let face = Color::TrueColor {
        r: 236,
        g: 190,
        b: 130,
    };
    let wing = Color::TrueColor {
        r: 190,
        g: 150,
        b: 225,
    };
    let body = Color::TrueColor {
        r: 158,
        g: 128,
        b: 210,
    };
    let star = Color::BrightBlack;

    vec![
        ("      .  ✦  .      ", star),
        ("        ___        ", amber),
        ("       (   )       ", amber),
        ("       (o‿o)       ", face),
        ("    ___/     \\___    ", wing),
        ("   /  /       \\  \\   ", wing),
        ("  (  (    ♥    )  )  ", wing),
        ("   \\  \\       /  /   ", wing),
        ("    \\  `.___.'  /    ", body),
        ("       \\_____/       ", body),
    ]
}

fn header_lines(ts: &str) -> Vec<String> {
    let heart = "♥".truecolor(240, 96, 140);
    vec![
        String::new(),
        format!("{}", "✨ Joséphine".truecolor(238, 108, 170).bold()),
        format!("{}", "Votre ange gardien système".truecolor(178, 148, 224)),
        String::new(),
        "Je veille sur votre machine et vous préviens".to_string(),
        format!("quand quelque chose ne va pas. {heart}"),
        String::new(),
        format!("{}", format!("Dernière vérification : {ts}").dimmed()),
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
        label: "Charge système",
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
        "cpu" => ("🖥️", "Utilisation CPU"),
        "memory" => ("🧠", "Mémoire"),
        "disk" => ("💽", "Espace disque"),
        "temperature" => ("🌡️", "Température"),
        "systemd" => ("🛡️", "Services critiques"),
        "updates" => ("🔄", "Mises à jour"),
        _ => ("•", "Système"),
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
        Severity::Attention => "[!] ATTENTION",
        Severity::Critique => "[✗] CRITIQUE",
    }
}

// ---------------------------------------------------------------------------
// Advice box
// ---------------------------------------------------------------------------

fn print_advice(global: Severity) {
    let message = match global {
        Severity::Info => {
            "Tout roule — votre machine file un parfait bonheur. Belle journée à vous !"
        }
        Severity::Attention => {
            "Rien de grave pour le moment, mais gardons un œil sur certaines choses."
        }
        Severity::Critique => {
            "Un point mérite votre attention — `josephine doctor` vous dira tout."
        }
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

/// Render one angel line padded to `ART_WIDTH`; any `♥` is always painted pink,
/// even when the surrounding line is a different colour.
fn render_art_line(text: &str, color: Color) -> String {
    let padded = pad(text, ART_WIDTH);
    match padded.find('♥') {
        Some(idx) => {
            let (pre, rest) = padded.split_at(idx);
            let post = &rest['♥'.len_utf8()..];
            format!(
                "{}{}{}",
                pre.color(color),
                "♥".truecolor(240, 96, 140),
                post.color(color)
            )
        }
        None => padded.color(color).to_string(),
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
