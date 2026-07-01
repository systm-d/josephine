//! `josephine report` — a plain-text, dated snapshot of the machine's health,
//! printed to stdout or written to a file for archiving.

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
use josephine_core::check::{CheckResult, Severity};
use josephine_core::config::Config;
use josephine_core::scheduler::run_all_checks;

use crate::output::{check_label, format_metric_value, primary_metric};

pub fn run(output: Option<PathBuf>) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_all_checks(&config)?;

    let generated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let report = render_report(&results, &generated, &hostname());

    match output {
        Some(path) => {
            std::fs::write(&path, &report)
                .with_context(|| format!("écriture de {}", path.display()))?;
            println!(
                "✨ Rapport déposé dans {} — un carnet de bord tout frais.",
                path.display()
            );
        }
        None => print!("{report}"),
    }
    Ok(())
}

fn render_report(results: &[CheckResult], generated: &str, host: &str) -> String {
    let global = results
        .iter()
        .map(CheckResult::worst_severity)
        .max()
        .unwrap_or(Severity::Info);

    let mut out = String::new();
    out.push_str("Joséphine — rapport système\n");
    out.push_str(&format!("Date        : {generated}\n"));
    out.push_str(&format!("Machine     : {host}\n"));
    out.push_str(&format!("État global : {}\n", state_label(global)));
    out.push_str(&"=".repeat(60));
    out.push('\n');

    for result in results {
        let severity = result.worst_severity();
        let value = result
            .status_value
            .clone()
            .or_else(|| primary_metric(result).map(format_metric_value))
            .unwrap_or_else(|| "—".to_string());

        out.push_str(&format!(
            "\n[{}] {} — {}\n",
            state_label(severity),
            check_label(&result.check_name),
            value
        ));
        for detail in &result.details {
            out.push_str(&format!("    {}\n", detail.trim()));
        }
    }

    out.push_str("\n------------------------------------------------------------\n");
    out.push_str("Généré par Joséphine · 100 % local\n");
    out
}

fn state_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "OK       ",
        Severity::Attention => "ATTENTION",
        Severity::Critique => "CRITIQUE ",
    }
}

fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "machine".to_string())
}
