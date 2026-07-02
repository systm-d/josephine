use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::UpdatesCheckConfig;
use crate::i18n::{self, Lang};

pub struct UpdatesCheck {
    config: UpdatesCheckConfig,
}

impl UpdatesCheck {
    pub fn new(config: UpdatesCheckConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Manager {
    Dnf,
    Apt,
    Pacman,
}

impl Manager {
    fn label(self) -> &'static str {
        match self {
            Manager::Dnf => "dnf",
            Manager::Apt => "apt",
            Manager::Pacman => "pacman",
        }
    }
}

impl Check for UpdatesCheck {
    fn name(&self) -> &str {
        "updates"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let manager = detect_manager();

        let (count, mut names) = match manager {
            Some(m) => count_updates(m),
            None => (0, Vec::new()),
        };

        let mut details = Vec::new();
        match manager {
            Some(m) => details.push(match i18n::lang() {
                Lang::En => format!("Manager: {}", m.label()),
                Lang::Fr => format!("Gestionnaire : {}", m.label()),
            }),
            None => details.push(
                i18n::t(
                    "No recognised package manager (apt/dnf/pacman).",
                    "Aucun gestionnaire de paquets reconnu (apt/dnf/pacman).",
                )
                .into(),
            ),
        }
        details.push(match i18n::lang() {
            Lang::En => format!("Available updates: {count}"),
            Lang::Fr => format!("Mises à jour disponibles : {count}"),
        });
        if !names.is_empty() {
            names.truncate(10);
            details.push(i18n::t("Packages concerned:", "Paquets concernés :").into());
            for name in &names {
                details.push(format!("  • {name}"));
            }
        }

        let status_value = match manager {
            None => i18n::t("Unknown manager", "Gestionnaire inconnu").to_string(),
            Some(_) if count == 0 => i18n::t("Up to date", "À jour").to_string(),
            Some(_) if count == 1 => {
                i18n::t("1 update available", "1 mise à jour disponible").to_string()
            }
            Some(_) => match i18n::lang() {
                Lang::En => format!("{count} updates available"),
                Lang::Fr => format!("{count} mises à jour disponibles"),
            },
        };

        Ok(CheckResult {
            check_name: "updates".into(),
            metrics: vec![Metric {
                name: "updates_available".into(),
                value: count as f64,
                unit: "updates".into(),
                threshold_warning: Some(self.config.warning),
                threshold_critical: Some(self.config.critical),
            }],
            details,
            top_processes: names,
            status_value: Some(status_value),
        })
    }
}

fn detect_manager() -> Option<Manager> {
    for (bin, manager) in [
        ("dnf", Manager::Dnf),
        ("apt", Manager::Apt),
        ("pacman", Manager::Pacman),
    ] {
        if command_exists(bin) {
            return Some(manager);
        }
    }
    None
}

fn command_exists(bin: &str) -> bool {
    Command::new(bin)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns `(count, package_names)`. Never fails: on any error the count is 0.
fn count_updates(manager: Manager) -> (usize, Vec<String>) {
    match manager {
        Manager::Dnf => run_and_parse(Command::new("dnf").args(["-q", "check-update"]), parse_dnf),
        Manager::Apt => run_and_parse(
            Command::new("apt").args(["list", "--upgradable"]),
            parse_apt,
        ),
        Manager::Pacman => {
            // `checkupdates` (pacman-contrib) is safe & doesn't need root; fall back to `pacman -Qu`.
            if command_exists("checkupdates") {
                run_and_parse(&mut Command::new("checkupdates"), parse_pacman)
            } else {
                run_and_parse(Command::new("pacman").arg("-Qu"), parse_pacman)
            }
        }
    }
}

fn run_and_parse(cmd: &mut Command, parse: fn(&str) -> Vec<String>) -> (usize, Vec<String>) {
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let names = parse(&stdout);
            (names.len(), names)
        }
        Err(_) => (0, Vec::new()),
    }
}

fn parse_dnf(stdout: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // dnf appends an "Obsoleting Packages" section we don't want to count.
        if trimmed.starts_with("Obsoleting") {
            break;
        }
        let mut cols = trimmed.split_whitespace();
        if let Some(first) = cols.next()
            && first.contains('.')
            && cols.count() >= 2
        {
            names.push(first.to_string());
        }
    }
    names
}

fn parse_apt(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| l.contains("[upgradable"))
        .filter_map(|l| l.split('/').next())
        .map(|s| s.trim().to_string())
        .collect()
}

fn parse_pacman(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| l.split_whitespace().next())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dnf_counts_package_lines_and_ignores_obsoleting() {
        let sample = "\nfoo.x86_64        1.2-3.fc43     updates\nbar.noarch        2.0-1.fc43     fedora\n\nObsoleting Packages\nbaz.x86_64        9.9            updates\n";
        let names = parse_dnf(sample);
        assert_eq!(names, vec!["foo.x86_64", "bar.noarch"]);
    }

    #[test]
    fn parse_apt_counts_upgradable_lines() {
        let sample = "Listing...\nfoo/stable 1.2 amd64 [upgradable from: 1.1]\nbar/stable 3.0 amd64 [upgradable from: 2.9]\n";
        assert_eq!(parse_apt(sample), vec!["foo", "bar"]);
    }

    #[test]
    fn parse_pacman_counts_nonempty_lines() {
        let sample = "linux 6.9 -> 6.10\nfirefox 1 -> 2\n";
        assert_eq!(parse_pacman(sample), vec!["linux", "firefox"]);
    }
}
