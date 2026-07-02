use anyhow::Result;
use sysinfo::{ProcessesToUpdate, System};

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;
use crate::i18n::{self, Lang};

pub struct CpuCheck {
    thresholds: CheckThresholds,
    system: System,
}

impl CpuCheck {
    pub fn new(thresholds: CheckThresholds) -> Self {
        Self {
            thresholds,
            system: System::new_all(),
        }
    }
}

impl Check for CpuCheck {
    fn name(&self) -> &str {
        "cpu"
    }

    fn run(&mut self) -> Result<CheckResult> {
        self.system.refresh_cpu_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.refresh_cpu_all();

        let usage = self.system.global_cpu_usage() as f64;
        let load = System::load_average();

        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top_processes: Vec<String> = processes
            .iter()
            .take(10)
            .map(|p| {
                format!(
                    "{} (PID {}) — {:.1} %",
                    p.name().to_string_lossy(),
                    p.pid(),
                    p.cpu_usage()
                )
            })
            .collect();

        let details = vec![
            match i18n::lang() {
                Lang::En => format!("CPU usage: {usage:.1} %"),
                Lang::Fr => format!("Utilisation CPU : {usage:.1} %"),
            },
            match i18n::lang() {
                Lang::En => format!(
                    "Load average: {:.2} / {:.2} / {:.2} (1/5/15 min)",
                    load.one, load.five, load.fifteen
                ),
                Lang::Fr => format!(
                    "Charge moyenne : {:.2} / {:.2} / {:.2} (1/5/15 min)",
                    load.one, load.five, load.fifteen
                ),
            },
        ];

        Ok(CheckResult {
            check_name: "cpu".into(),
            metrics: vec![Metric {
                name: "usage_percent".into(),
                value: usage,
                unit: "%".into(),
                threshold_warning: Some(self.thresholds.warning),
                threshold_critical: Some(self.thresholds.critical),
            }],
            details,
            top_processes,
            status_value: Some(format!("{usage:.0}%")),
        })
    }
}
