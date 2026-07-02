use anyhow::Result;
use sysinfo::{ProcessesToUpdate, System};

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;
use crate::i18n::{self, Lang};

pub struct MemoryCheck {
    thresholds: CheckThresholds,
    system: System,
}

impl MemoryCheck {
    pub fn new(thresholds: CheckThresholds) -> Self {
        Self {
            thresholds,
            system: System::new_all(),
        }
    }
}

impl Check for MemoryCheck {
    fn name(&self) -> &str {
        "memory"
    }

    fn run(&mut self) -> Result<CheckResult> {
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let total = self.system.total_memory() as f64;
        let used = self.system.used_memory() as f64;
        let usage_percent = if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        };

        let swap_total = self.system.total_swap() as f64;
        let swap_used = self.system.used_swap() as f64;
        let swap_percent = if swap_total > 0.0 {
            (swap_used / swap_total) * 100.0
        } else {
            0.0
        };

        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by_key(|b| std::cmp::Reverse(b.memory()));

        let top_processes: Vec<String> = processes
            .iter()
            .take(10)
            .map(|p| {
                format!(
                    "{} (PID {}) — {:.1} {}",
                    p.name().to_string_lossy(),
                    p.pid(),
                    p.memory() as f64 / 1_048_576.0,
                    i18n::t("MB", "Mo")
                )
            })
            .collect();

        let used_gb = used / 1_073_741_824.0;
        let total_gb = total / 1_073_741_824.0;
        let details = vec![
            match i18n::lang() {
                Lang::En => {
                    format!("Memory used: {usage_percent:.1} % ({used_gb:.1} / {total_gb:.1} GB)")
                }
                Lang::Fr => format!(
                    "Mémoire utilisée : {usage_percent:.1} % ({used_gb:.1} / {total_gb:.1} Go)"
                ),
            },
            match i18n::lang() {
                Lang::En => format!("Swap used: {swap_percent:.1} %"),
                Lang::Fr => format!("Swap utilisé : {swap_percent:.1} %"),
            },
        ];

        Ok(CheckResult {
            check_name: "memory".into(),
            metrics: vec![
                Metric {
                    name: "usage_percent".into(),
                    value: usage_percent,
                    unit: "%".into(),
                    threshold_warning: Some(self.thresholds.warning),
                    threshold_critical: Some(self.thresholds.critical),
                },
                Metric {
                    name: "swap_percent".into(),
                    value: swap_percent,
                    unit: "%".into(),
                    threshold_warning: Some(self.thresholds.warning),
                    threshold_critical: Some(self.thresholds.critical),
                },
            ],
            details,
            top_processes,
            status_value: Some(format!(
                "{usage_percent:.0}% ({} / {})",
                crate::check::human_size(used),
                crate::check::human_size(total)
            )),
        })
    }
}
