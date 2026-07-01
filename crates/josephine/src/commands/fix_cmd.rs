//! `josephine fix` — guided remediation. Joséphine spots what's wrong (failed
//! services, a tight disk) and shows the exact command to set it right. She
//! points the way; you keep the wheel — nothing privileged runs on its own.

use std::process::Command;

use anyhow::Result;
use josephine_core::check::Severity;
use josephine_core::config::Config;
use josephine_core::scheduler::run_all_checks;

pub fn run() -> Result<()> {
    println!("✨ Joséphine — corrections guidées\n");
    let mut findings = 0;

    // 1. Failed systemd units.
    let failed = failed_units();
    if failed.is_empty() {
        println!("• Services : tout tourne rond, personne au tapis.");
    } else {
        findings += failed.len();
        println!("• Services en échec :");
        for unit in &failed {
            println!("    - {unit}");
            println!("      → sudo systemctl restart {unit}");
        }
        println!("    (comprendre pourquoi : systemctl status <service>)");
    }

    // 2. Disk pressure, straight from the checks.
    let config = Config::load_default()?;
    let results = run_all_checks(&config)?;
    if let Some(disk) = results.iter().find(|r| r.check_name == "disk") {
        if disk.worst_severity() == Severity::Info {
            println!("• Disque : de l'air en réserve, rien à libérer dans l'urgence.");
        } else {
            findings += 1;
            println!("• Disque un peu à l'étroit :");
            for line in &disk.details {
                println!("    {}", line.trim());
            }
            println!("    → Faites le point : josephine clean");
        }
    }

    println!();
    if findings == 0 {
        println!("Rien à réparer — votre machine file un parfait bonheur. ✨");
    } else {
        println!(
            "Ces gestes restent entre vos mains : je montre le chemin, vous gardez le volant. ✨"
        );
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
