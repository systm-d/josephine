//! Inode check — a filesystem can be "full" on inodes while still showing free
//! space (lots of tiny files). Reads `df -iPT` (the `T` adds the filesystem
//! type, so we can skip read-only image mounts like snaps); runs fine as a
//! normal user.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;
use crate::i18n::{self, Lang};

pub struct InodeCheck {
    thresholds: CheckThresholds,
}

impl InodeCheck {
    pub fn new(thresholds: CheckThresholds) -> Self {
        Self { thresholds }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InodeReading {
    mount: String,
    usage_percent: f64,
}

impl Check for InodeCheck {
    fn name(&self) -> &str {
        "inode"
    }

    fn run(&mut self) -> Result<CheckResult> {
        Ok(build_result(&read_inode_usage(), &self.thresholds))
    }
}

fn build_result(readings: &[InodeReading], thresholds: &CheckThresholds) -> CheckResult {
    if readings.is_empty() {
        return CheckResult {
            check_name: "inode".into(),
            metrics: vec![],
            details: vec![
                i18n::t(
                    "Inode information unavailable.",
                    "Information sur les inodes indisponible.",
                )
                .into(),
            ],
            top_processes: vec![],
            status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
        };
    }

    let worst = readings
        .iter()
        .max_by(|a, b| a.usage_percent.total_cmp(&b.usage_percent))
        .expect("non-empty");

    let details = readings
        .iter()
        .map(|r| match i18n::lang() {
            Lang::En => format!("{}: {:.0} % inodes used", r.mount, r.usage_percent),
            Lang::Fr => format!("{} : {:.0} % d'inodes utilisés", r.mount, r.usage_percent),
        })
        .collect();

    CheckResult {
        check_name: "inode".into(),
        metrics: vec![Metric {
            name: "inode_usage_percent_worst".into(),
            value: worst.usage_percent,
            unit: "%".into(),
            threshold_warning: Some(thresholds.warning),
            threshold_critical: Some(thresholds.critical),
        }],
        details,
        top_processes: vec![],
        status_value: Some(match i18n::lang() {
            Lang::En => format!("{:.0}% of “{}”", worst.usage_percent, worst.mount),
            Lang::Fr => format!("{:.0}% de « {} »", worst.usage_percent, worst.mount),
        }),
    }
}

fn read_inode_usage() -> Vec<InodeReading> {
    match Command::new("df").args(["-iPT"]).output() {
        Ok(output) if output.status.success() => {
            parse_df_inodes(&String::from_utf8_lossy(&output.stdout))
        }
        _ => Vec::new(),
    }
}

/// Parse `df -iPT` output, keeping only real, writable filesystems with inode
/// accounting. Columns are: Filesystem, Type, Inodes, IUsed, IFree, IUse%,
/// Mounted on.
fn parse_df_inodes(stdout: &str) -> Vec<InodeReading> {
    stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 7 {
                return None;
            }
            let fstype = fields[1];
            let mount = fields[6];
            if is_pseudo_mount(mount) || is_readonly_image_fs(fstype) {
                return None;
            }
            // Filesystems without inode concept report "-" for the counts.
            let usage_percent: f64 = fields[5].trim_end_matches('%').parse().ok()?;
            Some(InodeReading {
                mount: mount.to_string(),
                usage_percent,
            })
        })
        .collect()
}

fn is_pseudo_mount(mount: &str) -> bool {
    mount.starts_with("/dev")
        || mount.starts_with("/proc")
        || mount.starts_with("/sys")
        || mount.starts_with("/run")
        || mount.starts_with("/snap")
}

/// Read-only image filesystems (snaps, AppImages, mounted ISOs) are packed to
/// exactly their file count, so they *always* report 100 % inode usage. That's
/// by design and never actionable — and since snapd mounts them under
/// `/var/lib/snapd/snap/…` (not `/snap`) on many distros, matching by path
/// misses them. Filtering by type catches every one, wherever it mounts, so a
/// few dozen snaps can't bury the real writable filesystems under false alarms.
fn is_readonly_image_fs(fstype: &str) -> bool {
    matches!(fstype, "squashfs" | "iso9660" | "erofs")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thresholds() -> CheckThresholds {
        CheckThresholds {
            enabled: true,
            interval_secs: 300,
            warning: 85.0,
            critical: 95.0,
        }
    }

    const SAMPLE: &str = "\
Filesystem     Type       Inodes  IUsed   IFree IUse% Mounted on
/dev/sda2      ext4      6111232 512345 5598887    9% /
tmpfs          tmpfs     4055820    123 4055697    1% /dev/shm
/dev/sda3      ext4     12222464 300000 11922464    3% /home
btrfsdev       btrfs           -      -       -     -  /data
/dev/loop0     squashfs    23329  23329       0  100% /var/lib/snapd/snap/core24/1587
/dev/loop1     squashfs       29     29       0  100% /snap/bare/5
";

    #[test]
    fn parses_real_filesystems_only() {
        let readings = parse_df_inodes(SAMPLE);
        let mounts: Vec<&str> = readings.iter().map(|r| r.mount.as_str()).collect();
        assert_eq!(mounts, vec!["/", "/home"]);
    }

    #[test]
    fn skips_read_only_image_filesystems() {
        // Snaps mount squashfs images that are always 100 % inodes by design.
        // They must never leak into the check — wherever snapd mounts them
        // (`/var/lib/snapd/snap/…` on most distros, not just `/snap`) — or they
        // bury the real filesystems under false criticals.
        let readings = parse_df_inodes(SAMPLE);
        assert!(readings.iter().all(|r| !r.mount.contains("snap")));
        assert!(readings.iter().all(|r| r.usage_percent < 100.0));
    }

    #[test]
    fn worst_drives_status_and_severity() {
        let result = build_result(&parse_df_inodes(SAMPLE), &thresholds());
        assert_eq!(result.status_value.as_deref(), Some("9% of “/”"));
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
    }

    #[test]
    fn high_inode_usage_is_critical() {
        let readings = vec![InodeReading {
            mount: "/".into(),
            usage_percent: 97.0,
        }];
        let result = build_result(&readings, &thresholds());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }
}
