use anyhow::Result;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::paths::Paths;
use josephine_core::storage::{EventRecord, Storage};

use crate::output::{check_label, sparkline};

/// `(check, metric, unit)` shown in the 24-hour trend table.
const TRACKED: &[(&str, &str, &str)] = &[
    ("cpu", "usage_percent", "%"),
    ("memory", "usage_percent", "%"),
    ("disk", "usage_percent_worst", "%"),
    ("temperature", "temp_max_celsius", "°C"),
    ("network", "gateway_latency_ms", "ms"),
    ("battery", "charge_percent", "%"),
];

pub fn run() -> Result<()> {
    let config = Config::load_default()?;
    if !config.history.enabled {
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

    crate::output::sober_header(Some(i18n::t("24 h", "24 h")), None);

    let mut trend = Table::new();
    trend.load_preset(UTF8_BORDERS_ONLY);
    trend.set_content_arrangement(ContentArrangement::Dynamic);
    trend.set_header(vec![
        Cell::new(i18n::t("Metric", "Métrique")).add_attribute(Attribute::Bold),
        Cell::new("Min").add_attribute(Attribute::Bold),
        Cell::new(i18n::t("Avg", "Moy")).add_attribute(Attribute::Bold),
        Cell::new("Max").add_attribute(Attribute::Bold),
        Cell::new(i18n::t("Trend (24 h)", "Tendance (24 h)")).add_attribute(Attribute::Bold),
    ]);

    let mut rows = 0;
    for (check, metric, unit) in TRACKED {
        if let Some(summary) = storage.metric_summary_24h(check, metric)? {
            trend.add_row(vec![
                Cell::new(check_label(check)),
                Cell::new(fmt_stat(summary.min, unit)),
                Cell::new(fmt_stat(summary.avg, unit)),
                Cell::new(fmt_stat(summary.max, unit)),
                Cell::new(sparkline(&summary.series)),
            ]);
            rows += 1;
        }
    }

    if rows == 0 {
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

    let events = storage.recent_events(10)?;
    if events.is_empty() {
        println!(
            "{}",
            i18n::t(
                "No events — a calm 24 hours.",
                "Aucun événement — 24 h calmes."
            )
        );
        return Ok(());
    }

    let mut events_table = Table::new();
    events_table.load_preset(UTF8_BORDERS_ONLY);
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
            Cell::new(format!("{} → {}", event.from_state, event.to_state)),
            Cell::new(format_event_value(event)),
        ]);
    }
    println!("{events_table}\n");
    Ok(())
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
