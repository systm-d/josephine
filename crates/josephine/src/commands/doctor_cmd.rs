use anyhow::Result;
use josephine_core::config::Config;

use crate::output::{print_checks_json, print_doctor, run_checks_with_progress};

pub fn run(verbose: bool, json: bool) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    if json {
        print_checks_json(&results);
        return Ok(());
    }
    print_doctor(&results, &config, verbose);
    Ok(())
}
