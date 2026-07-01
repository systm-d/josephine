//! Network check — stays strictly local: it pings the LAN default gateway and
//! reads local files (`/proc/net/route`, `/sys/class/net`, `/etc/resolv.conf`).
//! No external host is contacted, honouring Joséphine's "100 % local" rule.

use std::fs;
use std::net::Ipv4Addr;
use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::NetworkCheckConfig;

pub struct NetworkCheck {
    config: NetworkCheckConfig,
}

impl NetworkCheck {
    pub fn new(config: NetworkCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for NetworkCheck {
    fn name(&self) -> &str {
        "network"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let gateway = read_default_gateway();
        let interfaces = up_interfaces();
        let nameservers = read_nameservers();

        // Latency drives severity; an unreachable/absent gateway maps to the
        // warning level (offline or ICMP-filtered is a nudge, not a panic).
        let (value, status_value, headline) = match gateway {
            Some(gw) => match ping_latency_ms(gw) {
                Some(ms) => (
                    ms,
                    format!("{ms:.0} ms (passerelle)"),
                    format!("Passerelle {gw} — {ms:.1} ms"),
                ),
                None => (
                    self.config.warning,
                    "Passerelle injoignable".to_string(),
                    format!("Passerelle {gw} injoignable (ICMP filtré ou hors ligne ?)"),
                ),
            },
            None => (
                self.config.warning,
                "Hors ligne".to_string(),
                "Aucune passerelle par défaut (machine hors ligne ?).".to_string(),
            ),
        };

        let mut details = vec![headline];
        if interfaces.is_empty() {
            details.push("Aucune interface active hors loopback.".into());
        } else {
            details.push(format!("Interfaces actives : {}", interfaces.join(", ")));
        }
        if !nameservers.is_empty() {
            details.push(format!("DNS configurés : {}", nameservers.join(", ")));
        }

        Ok(CheckResult {
            check_name: "network".into(),
            metrics: vec![Metric {
                name: "gateway_latency_ms".into(),
                value,
                unit: "ms".into(),
                threshold_warning: Some(self.config.warning),
                threshold_critical: Some(self.config.critical),
            }],
            details,
            top_processes: vec![],
            status_value: Some(status_value),
        })
    }
}

fn read_default_gateway() -> Option<Ipv4Addr> {
    let content = fs::read_to_string("/proc/net/route").ok()?;
    parse_default_gateway(&content)
}

/// Parse the default-route gateway from `/proc/net/route` (hex, little-endian).
fn parse_default_gateway(content: &str) -> Option<Ipv4Addr> {
    for line in content.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let _iface = fields.next()?;
        let destination = fields.next()?;
        let gateway = fields.next()?;
        if destination == "00000000" {
            let raw = u32::from_str_radix(gateway, 16).ok()?;
            if raw == 0 {
                continue;
            }
            return Some(Ipv4Addr::from(raw.to_le_bytes()));
        }
    }
    None
}

fn up_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return interfaces;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" {
            continue;
        }
        let state = fs::read_to_string(entry.path().join("operstate")).unwrap_or_default();
        if state.trim() == "up" {
            interfaces.push(name);
        }
    }
    interfaces.sort();
    interfaces
}

fn read_nameservers() -> Vec<String> {
    fs::read_to_string("/etc/resolv.conf")
        .map(|content| parse_nameservers(&content))
        .unwrap_or_default()
}

/// Extract `nameserver` entries from a `resolv.conf`-style file.
fn parse_nameservers(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("nameserver"))
        .map(|rest| rest.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn ping_latency_ms(ip: Ipv4Addr) -> Option<f64> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1"])
        .arg(ip.to_string())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_ping_time(&String::from_utf8_lossy(&output.stdout))
}

/// Pull the `time=…` round-trip value (in ms) out of `ping` output.
fn parse_ping_time(stdout: &str) -> Option<f64> {
    let start = stdout.find("time=")? + "time=".len();
    let rest = &stdout[start..];
    let number: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    number.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_gateway_little_endian() {
        // Destination 0.0.0.0, gateway 0101A8C0 (little-endian) = 192.168.1.1.
        let sample = "Iface\tDestination\tGateway\tFlags\n\
                      wlan0\t00000000\t0101A8C0\t0003\n\
                      wlan0\t0001A8C0\t00000000\t0001\n";
        assert_eq!(
            parse_default_gateway(sample),
            Some(Ipv4Addr::new(192, 168, 1, 1))
        );
    }

    #[test]
    fn no_default_route_yields_none() {
        let sample = "Iface\tDestination\tGateway\tFlags\n\
                      wlan0\t0001A8C0\t00000000\t0001\n";
        assert_eq!(parse_default_gateway(sample), None);
    }

    #[test]
    fn parses_nameservers() {
        let sample = "# comment\nnameserver 127.0.0.53\nsearch lan\nnameserver 1.1.1.1\n";
        assert_eq!(parse_nameservers(sample), vec!["127.0.0.53", "1.1.1.1"]);
    }

    #[test]
    fn parses_ping_time() {
        let sample = "64 bytes from 192.168.1.1: icmp_seq=1 ttl=64 time=12.4 ms\n";
        assert_eq!(parse_ping_time(sample), Some(12.4));
    }
}
