use std::time::Duration;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use josephine_core::check::CheckResult;
use josephine_core::checks::build_checks;
use josephine_core::config::Config;
use josephine_core::i18n;

use super::style::{check_label, is_tty};

pub fn run_checks_with_progress(config: &Config) -> Result<Vec<CheckResult>> {
    let mut checks = build_checks(&config.checks);
    let total = checks.len();
    let mut results = Vec::with_capacity(total);

    if total == 0 {
        return Ok(results);
    }

    let pb = ProgressBar::new(total as u64);
    if is_tty() {
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan} {msg:<24} [{bar:28.cyan/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("█▓░"),
        );
        pb.enable_steady_tick(Duration::from_millis(80));
    } else {
        pb.set_draw_target(ProgressDrawTarget::hidden());
    }

    for (index, check) in checks.iter_mut().enumerate() {
        let label = check_label(check.name());
        pb.set_message(format!("{} {label}", i18n::t("Checking", "Analyse")));
        results.push(check.run()?);
        pb.set_position((index + 1) as u64);
    }

    pb.finish_and_clear();
    Ok(results)
}
