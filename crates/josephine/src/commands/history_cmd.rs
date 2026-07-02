use anyhow::Result;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use josephine_core::config::Config;
use josephine_core::paths::Paths;
use josephine_core::storage::{EventRecord, Storage};

use crate::output::{is_tty, print_banner, sparkline};

/// `(check, metric, label, unit)` shown in the 24-hour trend table.
const TRACKED: &[(&str, &str, &str, &str)] = &[
    ("cpu", "usage_percent", "CPU", "%"),
    ("memory", "usage_percent", "Mémoire", "%"),
    ("disk", "usage_percent_worst", "Disque", "%"),
    ("temperature", "temp_max_celsius", "Température", "°C"),
    ("network", "gateway_latency_ms", "Réseau", "ms"),
    ("battery", "charge_percent", "Batterie", "%"),
];

pub fn run() -> Result<()> {
    let config = Config::load_default()?;
    if !config.history.enabled {
        println!(
            "✨ Mon carnet de bord fait la sieste (historique désactivé). \
             Réveillez-le dans la configuration et je noterai tout."
        );
        return Ok(());
    }

    let paths = Paths::new()?;
    let storage = Storage::open(&paths)?;

    print_banner("Synthèse des dernières 24 heures");

    let mut trend = Table::new();
    trend.load_preset(UTF8_FULL);
    trend.set_content_arrangement(ContentArrangement::Dynamic);
    trend.set_header(vec![
        Cell::new("Métrique").add_attribute(Attribute::Bold),
        Cell::new("Min").add_attribute(Attribute::Bold),
        Cell::new("Moy").add_attribute(Attribute::Bold),
        Cell::new("Max").add_attribute(Attribute::Bold),
        Cell::new("Tendance (24 h)").add_attribute(Attribute::Bold),
    ]);

    let mut rows = 0;
    for (check, metric, label, unit) in TRACKED {
        if let Some(summary) = storage.metric_summary_24h(check, metric)? {
            trend.add_row(vec![
                Cell::new(label),
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
            "Pas encore de données à retracer. Lancez le démon \
             (`josephine daemon start`) et je remplirai ce carnet au fil des heures."
        );
        return Ok(());
    }
    println!("{trend}");
    println!();

    let events = storage.recent_events(10)?;
    if events.is_empty() {
        if is_tty() {
            println!("Rien à signaler — journée calme, comme je les aime. Je veille en silence.\n");
        } else {
            println!("Rien à signaler — journée calme, comme je les aime.\n");
        }
        return Ok(());
    }

    let mut events_table = Table::new();
    events_table.load_preset(UTF8_FULL);
    events_table.set_content_arrangement(ContentArrangement::Dynamic);
    events_table.set_header(vec![
        Cell::new("Heure").add_attribute(Attribute::Bold),
        Cell::new("Check").add_attribute(Attribute::Bold),
        Cell::new("Transition").add_attribute(Attribute::Bold),
        Cell::new("Valeur").add_attribute(Attribute::Bold),
    ]);
    for event in &events {
        events_table.add_row(vec![
            Cell::new(event.created_at.format("%H:%M").to_string()),
            Cell::new(&event.check_name),
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
