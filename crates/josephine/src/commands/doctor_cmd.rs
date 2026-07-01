use anyhow::Result;
use josephine_core::config::Config;

use crate::output::{print_doctor, run_checks_with_progress};

pub fn run(verbose: bool) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    print_doctor(&results, &config, verbose);
    Ok(())
}
