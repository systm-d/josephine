use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    ConfigAction, DaemonAction, NotifyAction, clean_cmd, config_cmd, daemon_cmd, doctor_cmd,
    fix_cmd, history_cmd, notify_cmd, report_cmd, status_cmd, update_cmd,
};

/// L'ange gardien de votre ordinateur
#[derive(Parser)]
#[command(
    name = "josephine",
    about = "L'ange gardien de votre ordinateur",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Mode interne — lancé par `josephine daemon start`
    #[arg(long = "__daemon__", hide = true)]
    daemon_internal: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Résumé rapide de l'état de la machine
    Status,
    /// Diagnostic complet
    Doctor {
        /// Rapport détaillé : seuils chiffrés, top 10 processus, intervalles
        #[arg(short, long)]
        verbose: bool,
    },
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
    /// Fait le point sur l'espace récupérable (aperçu par défaut)
    Clean {
        /// Nettoie réellement les miniatures au lieu du simple aperçu
        #[arg(long)]
        apply: bool,
    },
    /// Corrections guidées : ce qui cloche et comment y remédier
    Fix,
    /// Rapport système daté, à l'écran ou dans un fichier
    Report {
        /// Écrit le rapport dans ce fichier au lieu de l'afficher
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Notifications de bureau
    Notify {
        #[command(subcommand)]
        action: NotifyAction,
    },
    /// Vérifie et installe la dernière version de Joséphine
    Update {
        /// Signale une nouvelle version sans l'installer
        #[arg(long)]
        check: bool,
        /// N'attend pas de confirmation avant d'installer
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

/// Entry point: parse, dispatch, and map errors to a process exit code.
/// The warm French tone is intentional and preserved.
pub async fn run() -> ExitCode {
    match dispatch().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✨ Joséphine a rencontré un souci : {e}");
            ExitCode::from(1)
        }
    }
}

async fn dispatch() -> Result<()> {
    let cli = Cli::parse();

    if cli.daemon_internal {
        return josephine_core::daemon::run_daemon_foreground().await;
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
