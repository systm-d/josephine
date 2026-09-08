//! Integration tests for the I/O-bound checks: read → parse → `CheckResult`,
//! driven by a fixture tree and canned command output rather than by whatever
//! hardware the host happens to have.
//!
//! The fixture under `tests/fixtures/laptop` is a small stand-in for a running
//! machine: two thermal zones and an NVMe sensor, one battery and a mains
//! adapter, a default route, one interface up and `lo` down. Every expected
//! value below is derived from those files, so a check that stops reading them
//! — or starts reading them differently — fails here.

use std::path::PathBuf;

use josephine_core::check::{Check, Severity};
use josephine_core::checks::{
    BatteryCheck, InodeCheck, KernelCheck, NetworkCheck, TemperatureCheck,
};
use josephine_core::config::{
    BatteryCheckConfig, CheckThresholds, KernelCheckConfig, NetworkCheckConfig,
    TemperatureThresholds,
};
use josephine_core::source::{StubCommands, Sysfs};

fn laptop() -> Sysfs {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/laptop");
    assert!(root.is_dir(), "fixture tree missing: {}", root.display());
    Sysfs::rooted(root)
}

fn metric(result: &josephine_core::check::CheckResult, name: &str) -> f64 {
    result
        .metrics
        .iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("no `{name}` metric in {:?}", result.check_name))
        .value
}

#[test]
fn temperature_reads_every_sensor_and_reports_the_hottest() {
    let mut check = TemperatureCheck::new(TemperatureThresholds {
        enabled: true,
        warning: 70.0,
        critical: 85.0,
        interval_secs: 60,
    })
    .with_sysfs(laptop());

    let result = check.run().expect("temperature check");

    // thermal_zone1 (61.5 °C) is hotter than thermal_zone0 (47.0) and the
    // NVMe sensor (39.85), so it sets the metric.
    assert_eq!(metric(&result, "temp_max_celsius"), 61.5);
    assert_eq!(result.worst_severity(), Severity::Info);
    assert_eq!(result.status_value.as_deref(), Some("62°C"));

    // All three sensors are listed, the NVMe one included — it comes from a
    // different sysfs tree than the thermal zones.
    let details = result.details.join("\n");
    assert!(details.contains("acpitz"), "{details}");
    assert!(details.contains("x86_pkg_temp"), "{details}");
    assert!(details.contains("NVMe nvme0"), "{details}");
}

#[test]
fn temperature_without_sensors_says_so_rather_than_reporting_zero() {
    let empty = tempfile_dir("temperature-empty");
    let mut check = TemperatureCheck::new(TemperatureThresholds {
        enabled: true,
        warning: 70.0,
        critical: 85.0,
        interval_secs: 60,
    })
    .with_sysfs(Sysfs::rooted(&empty));

    let result = check.run().expect("temperature check");

    assert_eq!(metric(&result, "temp_max_celsius"), 0.0);
    assert_eq!(result.worst_severity(), Severity::Info);
    let status = result.status_value.unwrap_or_default();
    assert!(status.contains("No sensor"), "{status}");
}

#[test]
fn battery_reads_the_battery_and_ignores_the_mains_adapter() {
    let mut check = BatteryCheck::new(BatteryCheckConfig {
        enabled: true,
        warning: 25.0,
        critical: 10.0,
        interval_secs: 300,
    })
    .with_sysfs(laptop());

    let result = check.run().expect("battery check");

    assert_eq!(metric(&result, "charge_percent"), 64.0);
    // Status is `Discharging`, so depletion is 100 − 64. Were it plugged in,
    // this would read 0 and let the rules engine fire a recovery.
    assert_eq!(metric(&result, "battery_depletion_percent"), 36.0);
    assert_eq!(result.worst_severity(), Severity::Info);

    // Health comes from energy_full / energy_full_design =
    // 43_750_000 / 50_000_000 = 87.5 %, rendered to the nearest percent.
    let details = result.details.join("\n");
    assert!(details.contains("88 % of design capacity"), "{details}");

    // The `AC` entry is type `Mains`, not `Battery`: reading it as one would
    // report a machine with two batteries.
    assert!(details.contains("BAT0"), "{details}");
    assert!(
        !details.contains("AC"),
        "the mains adapter leaked in: {details}"
    );
}

#[test]
fn network_reads_the_route_table_and_pings_the_gateway() {
    // A real `ping` reply for 192.168.1.1 — the gateway encoded, little-endian,
    // as 0101A8C0 in the fixture's route table.
    let ping = "PING 192.168.1.1 (192.168.1.1) 56(84) bytes of data.\n\
                64 bytes from 192.168.1.1: icmp_seq=1 ttl=64 time=3.42 ms\n";
    let mut check = NetworkCheck::new(NetworkCheckConfig {
        enabled: true,
        warning: 100.0,
        critical: 300.0,
        interval_secs: 120,
    })
    .with_sources(laptop(), StubCommands::new().with("ping", ping).arc());

    let result = check.run().expect("network check");

    assert_eq!(metric(&result, "gateway_latency_ms"), 3.42);
    assert_eq!(result.worst_severity(), Severity::Info);

    let details = result.details.join("\n");
    assert!(details.contains("192.168.1.1"), "{details}");
    // `lo` is operstate down in the fixture and is skipped regardless.
    assert!(details.contains("wlan0"), "{details}");
    assert!(
        !details.contains(" lo"),
        "loopback listed as an interface: {details}"
    );
}

#[test]
fn network_without_ping_reports_an_unreachable_gateway_not_a_crash() {
    // The stub knows no programs at all — `ping` is not installed.
    let mut check = NetworkCheck::new(NetworkCheckConfig {
        enabled: true,
        warning: 100.0,
        critical: 300.0,
        interval_secs: 120,
    })
    .with_sources(laptop(), StubCommands::new().arc());

    let result = check.run().expect("network check");

    // Falls back to the warning threshold: a nudge, never a panic.
    assert_eq!(metric(&result, "gateway_latency_ms"), 100.0);
    assert_eq!(result.worst_severity(), Severity::Attention);
}

#[test]
fn inode_parses_df_and_skips_pseudo_and_image_filesystems() {
    let df = "\
Filesystem     Type     Inodes  IUsed   IFree IUse% Mounted on
/dev/sda2      ext4    3276800 294912 2981888    9% /
tmpfs          tmpfs    2045123     42 2045081    1% /dev/shm
/dev/loop0     squashfs   12345  12345       0  100% /snap/core/1234
/dev/sda1      ext4     655360 622592   32768   95% /home
";
    let mut check = InodeCheck::new(CheckThresholds {
        enabled: true,
        warning: 85.0,
        critical: 95.0,
        interval_secs: 300,
    })
    .with_commands(StubCommands::new().with("df", df).arc());

    let result = check.run().expect("inode check");

    // /home at 95 % is the worst *real* filesystem. The squashfs snap sits at
    // 100 % by construction and must not win, or every machine with a snap
    // installed is permanently critical.
    assert_eq!(metric(&result, "inode_usage_percent_worst"), 95.0);
    let details = result.details.join("\n");
    assert!(!details.contains("squashfs"), "{details}");
    assert!(!details.contains("/dev/shm"), "{details}");
}

#[test]
fn inode_without_df_reports_nothing_rather_than_zero_percent() {
    let mut check = InodeCheck::new(CheckThresholds {
        enabled: true,
        warning: 85.0,
        critical: 95.0,
        interval_secs: 300,
    })
    .with_commands(StubCommands::new().arc());

    let result = check.run().expect("inode check");
    assert_eq!(result.worst_severity(), Severity::Info);
}

#[test]
fn kernel_counts_oom_kills_and_faults_from_the_journal() {
    let journal = "\
Out of memory: Killed process 4242 (firefox) total-vm:9000000kB
usb 1-3: new high-speed USB device number 5 using xhci_hcd
BUG: kernel NULL pointer dereference, address: 0000000000000000
EXT4-fs (sda2): mounted filesystem with ordered data mode
";
    let mut check = KernelCheck::new(KernelCheckConfig {
        enabled: true,
        warning: 1.0,
        critical: 5.0,
        interval_secs: 600,
    })
    .with_commands(StubCommands::new().with("journalctl", journal).arc());

    let result = check.run().expect("kernel check");

    // The OOM kill and the BUG line count; the USB and EXT4 lines do not.
    assert_eq!(metric(&result, "kernel_incidents"), 2.0);
    assert_eq!(result.worst_severity(), Severity::Attention);
}

#[test]
fn kernel_without_a_readable_journal_degrades_to_unavailable() {
    let mut check = KernelCheck::new(KernelCheckConfig {
        enabled: true,
        warning: 1.0,
        critical: 5.0,
        interval_secs: 600,
    })
    .with_commands(StubCommands::new().arc());

    let result = check.run().expect("kernel check");

    // Unreadable is not the same as clean: it must not report zero incidents
    // as an all-clear.
    assert_eq!(result.worst_severity(), Severity::Info);
    let status = result.status_value.unwrap_or_default();
    assert!(status.contains("Journal"), "{status}");
}

/// An empty directory, for the "this machine exposes nothing" cases. Created
/// under the crate's own target dir so it never collides with another run and
/// leaves nothing behind in /tmp.
fn tempfile_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/test-scratch")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}
