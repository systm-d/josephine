mod cli;
mod commands;
mod output;
mod since;

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    cli::run().await
}
