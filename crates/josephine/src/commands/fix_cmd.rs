//! `josephine fix` — guided remediation. Joséphine spots what's wrong (failed
//! services, a tight disk) and shows the exact command to set it right. She
//! points the way; you keep the wheel — nothing privileged runs on its own.

use std::process::Command;

use anyhow::Result;
use josephine_core::check::Severity;
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::scheduler::run_all_checks;
use josephine_core::voice;

pub fn run() -> Result<()> {
    crate::output::sober_header(Some(i18n::t("fix", "fix")), Some(voice::fix_tagline()));
    let mut findings = 0;

    // 1. Failed systemd units.
    let failed = failed_units();
    if failed.is_empty() {
        println!(
            "{}",
            i18n::t(
                "• Services: all running smoothly, nobody down.",
                "• Services : tout tourne rond, personne au tapis.",
            )
        );
    } else {
        findings += failed.len();
        println!("{}", i18n::t("• Failed services:", "• Services en échec :"));
        for unit in &failed {
            println!("    - {unit}");
            println!("      → sudo systemctl restart {unit}");
        }
        println!(
            "{}",
            i18n::t(
                "    (find out why: systemctl status <service>)",
                "    (comprendre pourquoi : systemctl status <service>)",
            )
        );
    }

    // 2. Disk pressure, straight from the checks.
    let config = Config::load_default()?;
    let results = run_all_checks(&config)?;
    if let Some(disk) = results.iter().find(|r| r.check_name == "disk") {
        if disk.worst_severity() == Severity::Info {
            println!(
                "{}",
                i18n::t(
                    "• Disk: room to spare, nothing to free urgently.",
                    "• Disque : de l'air en réserve, rien à libérer dans l'urgence.",
                )
            );
        } else {
            findings += 1;
            println!(
                "{}",
                i18n::t("• Disk a little tight:", "• Disque un peu à l'étroit :")
            );
            for line in &disk.details {
                println!("    {}", line.trim());
            }
            println!(
                "{}",
                i18n::t(
                    "    → Take stock: josephine clean",
                    "    → Faites le point : josephine clean",
                )
            );
        }
    }

    println!();
    if findings == 0 {
        println!("{}", voice::fix_all_good());
    } else {
        println!("{}", voice::fix_hands_off());
    }
    Ok(())
}

fn failed_units() -> Vec<String> {
    let output = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain", "--no-pager"])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            parse_failed_units(&String::from_utf8_lossy(&output.stdout))
        }
        _ => Vec::new(),
    }
}

/// First column of each `systemctl --failed --plain` line is the unit name.
fn parse_failed_units(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| token.contains('.'))
        .map(|token| token.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_failed_unit_names() {
        let sample = "  nginx.service   loaded failed failed A web server\n\
                      backup.timer     loaded failed failed Nightly backup\n";
        assert_eq!(
            parse_failed_units(sample),
            vec!["nginx.service", "backup.timer"]
        );
    }

    #[test]
    fn empty_when_nothing_failed() {
        assert!(parse_failed_units("").is_empty());
    }
}
