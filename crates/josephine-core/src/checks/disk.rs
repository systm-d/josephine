use anyhow::Result;
use sysinfo::{DiskKind, Disks};

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;
use crate::i18n::{self, Lang};

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
        let mut worst_used = 0.0_f64;
        let mut worst_total = 0.0_f64;

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

            let used_gb = used / 1_073_741_824.0;
            let total_gb = total / 1_073_741_824.0;
            details.push(match i18n::lang() {
                Lang::En => format!("{mount} ({fs}): {usage_percent:.1} % used ({used_gb:.1} / {total_gb:.1} GB)"),
                Lang::Fr => format!("{mount} ({fs}) : {usage_percent:.1} % utilisé ({used_gb:.1} / {total_gb:.1} Go)"),
            });

            if disk.kind() == DiskKind::SSD {
                details.push(match i18n::lang() {
                    Lang::En => format!("  └ SSD detected on {mount}"),
                    Lang::Fr => format!("  └ SSD détecté sur {mount}"),
                });
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
                worst_used = used;
                worst_total = total;
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
            details.push(
                i18n::t(
                    "No mounted partition detected.",
                    "Aucune partition montée détectée.",
                )
                .into(),
            );
        }

        let status_value = if worst_total > 0.0 {
            let u = crate::check::human_size(worst_used);
            let t = crate::check::human_size(worst_total);
            match i18n::lang() {
                Lang::En => format!("{worst_usage:.0}% of “{worst_mount}” ({u} / {t})"),
                Lang::Fr => format!("{worst_usage:.0}% de « {worst_mount} » ({u} / {t})"),
            }
        } else {
            i18n::t("No partition", "Aucune partition").into()
        };

        let fullest = match i18n::lang() {
            Lang::En => format!("Fullest partition: {worst_mount} ({worst_usage:.1} %)"),
            Lang::Fr => format!("Partition la plus remplie : {worst_mount} ({worst_usage:.1} %)"),
        };

        Ok(CheckResult {
            check_name: "disk".into(),
            metrics,
            details,
            top_processes: vec![fullest],
            status_value: Some(status_value),
        })
    }
}
