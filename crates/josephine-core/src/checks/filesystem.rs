//! Filesystem check — a writable filesystem silently remounted **read-only**
//! is a classic early sign of a failing disk or corruption. Reads
//! `/proc/mounts`; degrades gracefully if it can't be read (should never
//! happen on Linux, but we don't want a false alarm if it ever does).

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::CheckThresholds;
use crate::i18n::{self, Lang};

/// Filesystem types that are normally mounted read-write. A mount using one
/// of these types with the `ro` option set is almost certainly a remount
/// forced by the kernel after detecting corruption or I/O errors — not by
/// design (unlike `squashfs`/`iso9660`/`erofs` image mounts, which are
/// deliberately read-only).
const WRITABLE_FSTYPES: &[&str] = &[
    "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "vfat", "exfat", "jfs", "reiserfs", "ntfs3",
];

pub struct FilesystemCheck {
    thresholds: CheckThresholds,
}

impl FilesystemCheck {
    pub fn new(thresholds: CheckThresholds) -> Self {
        Self { thresholds }
    }
}

impl Check for FilesystemCheck {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
            return Ok(unavailable());
        };
        Ok(build_result(
            &find_readonly_mounts(&content),
            &self.thresholds,
        ))
    }
}

fn build_result(flagged: &[String], thresholds: &CheckThresholds) -> CheckResult {
    let count = flagged.len();

    let status_value = match (i18n::lang(), count) {
        (Lang::En, 0) => "all read-write".to_string(),
        (Lang::En, n) => format!("{n} read-only: “{}”", flagged[0]),
        (Lang::Fr, 0) => "tout est accessible en écriture".to_string(),
        (Lang::Fr, n) => format!("{n} en lecture seule : « {} »", flagged[0]),
    };

    let mut details: Vec<String> = flagged
        .iter()
        .map(|mount| match i18n::lang() {
            Lang::En => format!("“{mount}” is mounted read-only."),
            Lang::Fr => format!("« {mount} » est monté en lecture seule."),
        })
        .collect();
    if flagged.is_empty() {
        details.push(
            i18n::t(
                "All normally-writable filesystems are mounted read-write.",
                "Tous les systèmes de fichiers habituellement accessibles en \
                 écriture le sont bien.",
            )
            .into(),
        );
    }

    CheckResult {
        check_name: "filesystem".into(),
        metrics: vec![Metric {
            name: "readonly_mounts".into(),
            value: count as f64,
            unit: "mounts".into(),
            threshold_warning: Some(thresholds.warning),
            threshold_critical: Some(thresholds.critical),
        }],
        details,
        top_processes: vec![],
        status_value: Some(status_value),
    }
}

fn unavailable() -> CheckResult {
    CheckResult {
        check_name: "filesystem".into(),
        metrics: vec![],
        details: vec![
            i18n::t(
                "Mount list unreadable (`/proc/mounts`).",
                "Liste des montages illisible (`/proc/mounts`).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

/// Parse `/proc/mounts` (fields: device, mount point, fstype, options, …) and
/// return the mount points that are unexpectedly read-only: a real
/// read-write-class filesystem (not a pseudo mount, not an inherently
/// read-only image type) whose options contain the whole-word `ro` flag.
fn find_readonly_mounts(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 {
                return None;
            }
            let mount_point = fields[1];
            let fstype = fields[2];
            let options = fields[3];
            if is_pseudo_mount(mount_point) || !WRITABLE_FSTYPES.contains(&fstype) {
                return None;
            }
            has_ro_option(options).then(|| mount_point.to_string())
        })
        .collect()
}

/// Whole-word match on the comma-separated options field — avoids false
/// positives like `norecovery` or a hypothetical `rowhatever` option.
fn has_ro_option(options: &str) -> bool {
    options.split(',').any(|opt| opt == "ro")
}

fn is_pseudo_mount(mount_point: &str) -> bool {
    mount_point.starts_with("/proc")
        || mount_point.starts_with("/sys")
        || mount_point.starts_with("/run")
        || mount_point.starts_with("/dev")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thresholds() -> CheckThresholds {
        CheckThresholds {
            enabled: true,
            interval_secs: 120,
            warning: 1.0,
            critical: 1.0,
        }
    }

    const ONE_READONLY: &str = "\
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=1633080k,mode=755 0 0
/dev/sda1 / ext4 rw,relatime 0 0
/dev/sda2 /home ext4 ro,relatime 0 0
/dev/loop0 /snap/core20/1970 squashfs ro,nodev,relatime 0 0
";

    const ALL_READWRITE: &str = "\
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=1633080k,mode=755 0 0
/dev/sda1 / ext4 rw,relatime 0 0
/dev/sda2 /home ext4 rw,relatime 0 0
/dev/loop0 /snap/core20/1970 squashfs ro,nodev,relatime 0 0
";

    const SNAP_ONLY: &str = "\
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
/dev/loop0 /snap/core20/1970 squashfs ro,nodev,relatime 0 0
";

    #[test]
    fn flags_a_real_readonly_remount() {
        let flagged = find_readonly_mounts(ONE_READONLY);
        assert_eq!(flagged, vec!["/home".to_string()]);
    }

    #[test]
    fn all_read_write_flags_nothing() {
        let flagged = find_readonly_mounts(ALL_READWRITE);
        assert!(flagged.is_empty());
    }

    #[test]
    fn readonly_squashfs_snap_is_not_flagged() {
        let flagged = find_readonly_mounts(SNAP_ONLY);
        assert!(flagged.is_empty());
    }

    #[test]
    fn one_readonly_mount_is_critical() {
        let result = build_result(&find_readonly_mounts(ONE_READONLY), &thresholds());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
        assert_eq!(result.status_value.as_deref(), Some("1 read-only: “/home”"));
    }

    #[test]
    fn all_writable_is_info() {
        let result = build_result(&find_readonly_mounts(ALL_READWRITE), &thresholds());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(result.status_value.as_deref(), Some("all read-write"));
    }
}
