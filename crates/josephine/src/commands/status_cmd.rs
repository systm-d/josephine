use anyhow::Result;
use josephine_core::config::Config;

use crate::output::{print_checks_json, print_status_table, run_checks_with_progress};

pub fn run(json: bool) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    if json {
        print_checks_json(&results);
        return Ok(());
    }
    print_status_table(&results);
    Ok(())
}
