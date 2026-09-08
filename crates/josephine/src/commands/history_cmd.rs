use anyhow::{Result, bail};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::paths::Paths;
use josephine_core::storage::{EventRecord, MetricSummary, Storage};
use josephine_core::voice;

use crate::output::{check_label, sparkline};
use crate::since;

/// `(check, metric, unit)` shown in the 24-hour trend table.
const TRACKED: &[(&str, &str, &str)] = &[
    ("cpu", "usage_percent", "%"),
    ("memory", "usage_percent", "%"),
    ("disk", "usage_percent_worst", "%"),
    ("temperature", "temp_max_celsius", "°C"),
    ("network", "gateway_latency_ms", "ms"),
    ("battery", "charge_percent", "%"),
];

/// The default window, and the one `history` has always shown.
const DEFAULT_HOURS: i64 = 24;

/// How many events to list. Unchanged for the human view; the JSON export
/// carries the same cap so the two never disagree about what happened.
const EVENT_LIMIT: usize = 10;

pub fn run(since_spec: Option<String>, only: Vec<String>, json: bool) -> Result<()> {
    let hours = match since_spec.as_deref() {
        Some(spec) => since::parse_since(spec)?,
        None => DEFAULT_HOURS,
    };

    // A name nobody tracks would otherwise show an empty table and no reason.
    for name in &only {
        if !TRACKED.iter().any(|(check, _, _)| check == name) {
            let known: Vec<&str> = TRACKED.iter().map(|(check, _, _)| *check).collect();
            bail!(match i18n::lang() {
                i18n::Lang::En => format!(
                    "I keep no trend for `{name}`. I track: {}.",
                    known.join(", ")
                ),
                i18n::Lang::Fr => format!(
                    "Je ne tiens pas de tendance pour « {name} ». Je suis : {}.",
                    known.join(", ")
                ),
            });
        }
    }

    let config = Config::load_default()?;
    if !config.history.enabled {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&Export::disabled(hours))?
            );
            return Ok(());
        }
        println!(
            "{}",
            i18n::t(
                "History is off. Enable it in the config and I'll keep the log.",
                "Historique désactivé. Activez-le dans la configuration et je tiendrai le journal.",
            )
        );
        return Ok(());
    }

    let paths = Paths::new()?;
    let storage = Storage::open(&paths)?;

    let mut collected: Vec<(&str, &str, &str, MetricSummary)> = Vec::new();
    for (check, metric, unit) in TRACKED {
        if !only.is_empty() && !only.iter().any(|name| name == check) {
            continue;
        }
        if let Some(summary) = storage.metric_summary(check, metric, hours)? {
            collected.push((check, metric, unit, summary));
        }
    }

    let events = storage.events_since(hours, EVENT_LIMIT)?;

    if json {
        let export = Export::new(hours, &collected, &events);
        println!("{}", serde_json::to_string_pretty(&export)?);
        return Ok(());
    }

    let window = since::label(hours);

    crate::output::sober_header(
        Some(&window),
        Some(&match (i18n::lang(), hours) {
            (i18n::Lang::En, 24) => "The last 24 hours, at a glance.".to_string(),
            (i18n::Lang::En, _) => format!("The last {window}, at a glance."),
            (i18n::Lang::Fr, 24) => "Les dernières 24 heures, d'un coup d'œil.".to_string(),
            (i18n::Lang::Fr, _) => format!("Les dernières {window}, d'un coup d'œil."),
        }),
    );

    let mut trend = Table::new();
    trend.load_style(UTF8_BORDERS_ONLY);
    trend.set_content_arrangement(ContentArrangement::Dynamic);
    trend.set_header(vec![
        Cell::new(i18n::t("Metric", "Métrique")).add_attribute(Attribute::Bold),
        Cell::new("Min").add_attribute(Attribute::Bold),
        Cell::new(i18n::t("Avg", "Moy")).add_attribute(Attribute::Bold),
        Cell::new("Max").add_attribute(Attribute::Bold),
        Cell::new(match i18n::lang() {
            i18n::Lang::En => format!("Trend ({window})"),
            i18n::Lang::Fr => format!("Tendance ({window})"),
        })
        .add_attribute(Attribute::Bold),
    ]);

    for (check, _, unit, summary) in &collected {
        trend.add_row(vec![
            Cell::new(check_label(check)),
            Cell::new(fmt_stat(summary.min, unit)),
            Cell::new(fmt_stat(summary.avg, unit)),
            Cell::new(fmt_stat(summary.max, unit)),
            Cell::new(sparkline(&summary.series)),
        ]);
    }

    if collected.is_empty() {
        println!(
            "{}",
            i18n::t(
                "No data yet. Start the daemon (`josephine daemon start`) and it fills in over the hours.",
                "Pas encore de données. Lancez le démon (`josephine daemon start`) et il se remplit au fil des heures.",
            )
        );
        return Ok(());
    }
    println!("{trend}");
    println!();

    if events.is_empty() {
        println!("{}", voice::history_calm());
        return Ok(());
    }

    let mut events_table = Table::new();
    events_table.load_style(UTF8_BORDERS_ONLY);
    events_table.set_content_arrangement(ContentArrangement::Dynamic);
    events_table.set_header(vec![
        Cell::new(i18n::t("Time", "Heure")).add_attribute(Attribute::Bold),
        Cell::new("Check").add_attribute(Attribute::Bold),
        Cell::new("Transition").add_attribute(Attribute::Bold),
        Cell::new(i18n::t("Value", "Valeur")).add_attribute(Attribute::Bold),
    ]);
    for event in &events {
        events_table.add_row(vec![
            Cell::new(event.created_at.format("%H:%M").to_string()),
            Cell::new(check_label(&event.check_name)),
            Cell::new(format!(
                "{} → {}",
                state_phrase(&event.from_state),
                state_phrase(&event.to_state)
            )),
            Cell::new(format_event_value(event)),
        ]);
    }
    println!("{events_table}\n");
    println!("{}", voice::history_closing());
    Ok(())
}

/// The `--json` shape. Stable and documented in the README: a window, the
/// per-metric summaries, and the state changes inside it.
#[derive(serde::Serialize)]
struct Export {
    /// The window, in hours — the machine-readable half of `window`.
    window_hours: i64,
    /// The window as it reads to a person: `24 h`, `7 d`.
    window: String,
    /// `false` when history is switched off in the config, in which case the
    /// two lists below are empty rather than absent.
    enabled: bool,
    metrics: Vec<ExportMetric>,
    events: Vec<ExportEvent>,
}

#[derive(serde::Serialize)]
struct ExportMetric {
    check: String,
    metric: String,
    unit: String,
    min: f64,
    avg: f64,
    max: f64,
    /// Averaged buckets, oldest first — hourly up to 48 h, daily beyond.
    series: Vec<f64>,
}

#[derive(serde::Serialize)]
struct ExportEvent {
    check: String,
    metric: String,
    from: String,
    to: String,
    value: f64,
    message: String,
    /// RFC3339, as stored.
    at: String,
}

impl Export {
    fn new(
        hours: i64,
        metrics: &[(&str, &str, &str, MetricSummary)],
        events: &[EventRecord],
    ) -> Self {
        Self {
            window_hours: hours,
            window: since::label(hours),
            enabled: true,
            metrics: metrics
                .iter()
                .map(|(check, metric, unit, summary)| ExportMetric {
                    check: (*check).to_string(),
                    metric: (*metric).to_string(),
                    unit: (*unit).to_string(),
                    min: summary.min,
                    avg: summary.avg,
                    max: summary.max,
                    series: summary.series.clone(),
                })
                .collect(),
            events: events
                .iter()
                .map(|event| ExportEvent {
                    check: event.check_name.clone(),
                    metric: event.metric_name.clone(),
                    from: event.from_state.clone(),
                    to: event.to_state.clone(),
                    value: event.value,
                    message: event.message.clone(),
                    at: event.created_at.to_rfc3339(),
                })
                .collect(),
        }
    }

    /// History switched off: the same shape, honestly empty, so a script does
    /// not have to special-case a missing document.
    fn disabled(hours: i64) -> Self {
        Self {
            window_hours: hours,
            window: since::label(hours),
            enabled: false,
            metrics: Vec::new(),
            events: Vec::new(),
        }
    }
}

/// Soften a stored event state (`NORMAL` / `WARNING` / `CRITICAL` / `RECOVERED`)
/// into a calm glyph-and-word pair, matching the shape language used across
/// `status` and `doctor`. Plain glyphs (no colour) so the table stays aligned.
fn state_phrase(state: &str) -> String {
    let (glyph, word) = match state {
        "NORMAL" => ("●", i18n::t("calm", "au calme")),
        "WARNING" => ("▲", i18n::t("attention", "attention")),
        "CRITICAL" => ("✕", i18n::t("critical", "critique")),
        "RECOVERED" => ("●", i18n::t("resolved", "résolu")),
        other => ("·", other),
    };
    format!("{glyph} {word}")
}

fn fmt_stat(value: f64, unit: &str) -> String {
    match unit {
        "%" => format!("{value:.0} %"),
        "°C" => format!("{value:.0} °C"),
        "ms" => format!("{value:.0} ms"),
        _ => format!("{value:.0} {unit}"),
    }
}

fn format_event_value(event: &EventRecord) -> String {
    match event.check_name.as_str() {
        "temperature" => format!("{:.0} °C", event.value),
        "network" => format!("{:.0} ms", event.value),
        "systemd" if event.metric_name == "failed_units" => {
            format!("{:.0} service(s)", event.value)
        }
        "systemd" => format!("{:.0} restart(s)", event.value),
        _ => format!("{:.0} %", event.value),
    }
}
