mod commands;
mod output;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    config_cmd, daemon_cmd, doctor_cmd, history_cmd, status_cmd, stub_cmd, ConfigAction,
    DaemonAction, StubCommand,
};

#[derive(Parser)]
#[command(
    name = "josephine",
    about = "L'ange gardien de votre ordinateur",
    version,
    disable_version_flag = false
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Mode interne — lancé par `josephine daemon start`
    #[arg(long = "__daemon__", hide = true)]
    __daemon__: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Résumé rapide de l'état de la machine
    Status,
    /// Diagnostic complet
    Doctor,
    /// Historique des dernières 24 heures
    History,
    /// Gestion du démon de surveillance
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Nettoyage (bientôt)
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
    /// Corrections guidées (bientôt)
    Fix,
    /// Rapport complet (bientôt)
    Report,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✨ Joséphine a rencontré un souci : {e}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.__daemon__ {
        return josephine_core::daemon::run_daemon_foreground()
            .await
            .map_err(Into::into);
    }

    match cli.command {
        Some(Commands::Status) => status_cmd::run()?,
        Some(Commands::Doctor) => doctor_cmd::run()?,
        Some(Commands::History) => history_cmd::run()?,
        Some(Commands::Daemon { action }) => daemon_cmd::run(action).await?,
        Some(Commands::Config { action }) => config_cmd::run(action)?,
        Some(Commands::Clean { dry_run }) => stub_cmd::run(StubCommand::Clean { dry_run })?,
        Some(Commands::Fix) => stub_cmd::run(StubCommand::Fix)?,
        Some(Commands::Report) => stub_cmd::run(StubCommand::Report)?,
        None => {
            status_cmd::run()?;
        }
    }

    Ok(())
}
