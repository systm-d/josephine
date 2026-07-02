use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    ConfigAction, DaemonAction, NotifyAction, clean_cmd, config_cmd, daemon_cmd, doctor_cmd,
    fix_cmd, history_cmd, notify_cmd, report_cmd, status_cmd, update_cmd,
};

/// Your computer's guardian angel
#[derive(Parser)]
#[command(name = "josephine", about = "Your computer's guardian angel", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Internal mode — spawned by `josephine daemon start`
    #[arg(long = "__daemon__", hide = true)]
    daemon_internal: bool,
}

// clap help is always in English (the runtime language option covers command
// output; localising --help would mean building the Command tree dynamically).
#[derive(Subcommand)]
enum Commands {
    /// Quick summary of your machine's health
    Status,
    /// Full diagnostics
    Doctor {
        /// Detailed report: numeric thresholds, top 10 processes, intervals
        #[arg(short, long)]
        verbose: bool,
    },
    /// The last 24 hours
    History,
    /// Manage the monitoring daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Report reclaimable disk space (preview by default)
    Clean {
        /// Actually clear the thumbnail cache instead of just previewing
        #[arg(long)]
        apply: bool,
    },
    /// Guided fixes: what's wrong and how to remedy it
    Fix,
    /// Dated system report, to the screen or a file
    Report {
        /// Write the report to this file instead of printing it
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Desktop notifications
    Notify {
        #[command(subcommand)]
        action: NotifyAction,
    },
    /// Check for and install the latest version of Joséphine
    Update {
        /// Report a new version without installing it
        #[arg(long)]
        check: bool,
        /// Don't wait for confirmation before installing
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

/// Entry point: parse, dispatch, and map errors to a process exit code.
pub async fn run() -> ExitCode {
    use josephine_core::i18n::{self, Lang};
    match dispatch().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "{}",
                match i18n::lang() {
                    Lang::En => format!("✨ Joséphine ran into a snag: {e}"),
                    Lang::Fr => format!("✨ Joséphine a rencontré un souci : {e}"),
                }
            );
            ExitCode::from(1)
        }
    }
}

async fn dispatch() -> Result<()> {
    let cli = Cli::parse();

    if cli.daemon_internal {
        return josephine_core::daemon::run_daemon_foreground().await;
    }

    // Apply the configured language up front so every command renders alike.
    if let Ok(config) = josephine_core::config::Config::load_default() {
        josephine_core::i18n::set_lang(config.language);
    }

    match cli.command {
        Some(Commands::Status) => status_cmd::run()?,
        Some(Commands::Doctor { verbose }) => doctor_cmd::run(verbose)?,
        Some(Commands::History) => history_cmd::run()?,
        Some(Commands::Daemon { action }) => daemon_cmd::run(action).await?,
        Some(Commands::Config { action }) => config_cmd::run(action)?,
        Some(Commands::Clean { apply }) => clean_cmd::run(apply)?,
        Some(Commands::Fix) => fix_cmd::run()?,
        Some(Commands::Report { output }) => report_cmd::run(output)?,
        Some(Commands::Notify { action }) => notify_cmd::run(action)?,
        Some(Commands::Update { check, yes }) => update_cmd::run(check, yes)?,
        None => status_cmd::run()?,
    }

    Ok(())
}
