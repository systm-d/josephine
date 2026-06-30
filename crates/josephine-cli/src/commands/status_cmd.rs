use anyhow::Result;
use josephine_core::config::Config;

use crate::output::{print_status_table, run_checks_with_progress};

pub fn run() -> Result<()> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    print_status_table(&results);
    Ok(())
}
