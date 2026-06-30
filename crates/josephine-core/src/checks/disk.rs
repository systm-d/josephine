use anyhow::Result;
use sysinfo::{DiskKind, Disks};

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;

pub struct DiskCheck {
    thresholds: CheckThresholds,
    disks: Disks,
}

impl DiskCheck {
    pub fn new(thresholds: CheckThresholds) -> Self {
        Self {
            thresholds,
            disks: Disks::new_with_refreshed_list(),
        }
    }
}

fn skip_filesystem(name: &str) -> bool {
    matches!(
        name,
        "tmpfs" | "devtmpfs" | "squashfs" | "overlay" | "efivarfs" | "ramfs"
    )
}

impl Check for DiskCheck {
    fn name(&self) -> &str {
        "disk"
    }

    fn run(&mut self) -> Result<CheckResult> {
        self.disks.refresh(true);

        let mut metrics = Vec::new();
        let mut details = Vec::new();
        let mut worst_usage = 0.0_f64;
        let mut worst_mount = String::from("/");

        for disk in self.disks.list() {
            let fs = disk.file_system().to_string_lossy();
            if skip_filesystem(&fs) {
                continue;
            }

            let total = disk.total_space() as f64;
            if total == 0.0 {
                continue;
            }

            let available = disk.available_space() as f64;
            let used = total - available;
            let usage_percent = (used / total) * 100.0;
            let mount = disk.mount_point().to_string_lossy().to_string();

            details.push(format!(
                "{mount} ({fs}) : {:.1} % utilisé ({:.1} / {:.1} Go)",
                usage_percent,
                used / 1_073_741_824.0,
                total / 1_073_741_824.0
            ));

            if disk.kind() == DiskKind::SSD {
                details.push(format!("  └ SSD détecté sur {mount}"));
            }

            metrics.push(Metric {
                name: format!("usage_percent_{}", mount.replace('/', "_")),
                value: usage_percent,
                unit: "%".into(),
                threshold_warning: None,
                threshold_critical: None,
            });

            if usage_percent > worst_usage {
                worst_usage = usage_percent;
                worst_mount = mount;
            }
        }

        metrics.push(Metric {
            name: "usage_percent_worst".into(),
            value: worst_usage,
            unit: "%".into(),
            threshold_warning: Some(self.thresholds.warning),
            threshold_critical: Some(self.thresholds.critical),
        });

        if details.is_empty() {
            details.push("Aucune partition montée détectée.".into());
        }

        Ok(CheckResult {
            check_name: "disk".into(),
            metrics,
            details,
            top_processes: vec![format!(
                "Partition la plus remplie : {worst_mount} ({worst_usage:.1} %)"
            )],
        })
    }
}
