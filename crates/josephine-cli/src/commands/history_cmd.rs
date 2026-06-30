use anyhow::Result;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use josephine_core::config::Config;
use josephine_core::paths::Paths;
use josephine_core::storage::{EventRecord, Storage};

use crate::output::{is_tty, print_banner};

pub fn run() -> Result<()> {
    let config = Config::load_default()?;
    if !config.history.enabled {
        println!("L'historique est désactivé dans votre configuration.");
        return Ok(());
    }

    let paths = Paths::new()?;
    let storage = Storage::open(&paths)?;
    let summary = storage.history_last_24h()?;

    print_banner("Synthèse des dernières 24 heures");

    let mut metrics_table = Table::new();
    metrics_table.load_preset(UTF8_FULL);
    metrics_table.set_content_arrangement(ContentArrangement::Dynamic);
    metrics_table.set_header(vec![
        Cell::new("Métrique").add_attribute(Attribute::Bold),
        Cell::new("Maximum").add_attribute(Attribute::Bold),
    ]);
    metrics_table.add_row(vec![Cell::new("CPU"), Cell::new(format_max(summary.cpu_max, "%"))]);
    metrics_table.add_row(vec![Cell::new("RAM"), Cell::new(format_max(summary.memory_max, "%"))]);
    metrics_table.add_row(vec![
        Cell::new("Température"),
        Cell::new(format_max(summary.temperature_max, "°C")),
    ]);
    metrics_table.add_row(vec![
        Cell::new("Disque"),
        Cell::new(format_max(summary.disk_max, "%")),
    ]);
    println!("{metrics_table}");
    println!();

    if summary.recent_events.is_empty() {
        if is_tty() {
            println!("Aucun événement notable — Joséphine veille en silence.\n");
        } else {
            println!("Aucun événement notable.\n");
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

    for event in &summary.recent_events {
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

fn format_max(value: Option<f64>, unit: &str) -> String {
    match value {
        Some(v) => format!("{v:.0}{unit}"),
        None => "—".into(),
    }
}

fn format_event_value(event: &EventRecord) -> String {
    match event.check_name.as_str() {
        "temperature" => format!("{:.0} °C", event.value),
        "systemd" if event.metric_name == "failed_units" => {
            format!("{:.0} service(s)", event.value)
        }
        "systemd" => format!("{:.0} restart(s)", event.value),
        _ => format!("{:.0} %", event.value),
    }
}
