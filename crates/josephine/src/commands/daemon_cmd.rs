use anyhow::Result;
use clap::Subcommand;
use josephine_core::daemon::{DaemonControl, DaemonStatus};
use josephine_core::i18n;

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the monitoring daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart,
    /// Show the daemon's status
    Status,
    /// Show the latest logs
    Logs,
    /// Run the watcher in the foreground (used by the systemd `--user` unit)
    Run,
}

pub async fn run(action: DaemonAction) -> Result<()> {
    let exe = std::env::current_exe()?;
    let control = DaemonControl::new(exe)?;

    match action {
        DaemonAction::Start => {
            control.start()?;
            println!(
                "{}",
                i18n::t(
                    "Daemon started — on watch, eyes open. Go about your day.",
                    "Démon démarré — de garde, l'œil ouvert. Vaquez tranquille.",
                )
            );
        }
        DaemonAction::Stop => {
            control.stop()?;
            println!(
                "{}",
                i18n::t(
                    "Daemon stopped. Call me at the slightest trouble.",
                    "Démon arrêté. Appelez-moi au moindre souci.",
                )
            );
        }
        DaemonAction::Restart => {
            control.restart()?;
            println!(
                "{}",
                i18n::t(
                    "Daemon restarted — back on watch.",
                    "Démon redémarré — de nouveau de garde.",
                )
            );
        }
        DaemonAction::Status => match control.status()? {
            DaemonStatus::Running { pid, started_at } => {
                println!(
                    "{} (PID {pid})",
                    i18n::t(
                        "State: on watch, eyes open",
                        "État : de garde, l'œil ouvert"
                    )
                );
                if let Some(t) = started_at
                    && let Ok(elapsed) = t.elapsed()
                {
                    let mins = elapsed.as_secs() / 60;
                    println!(
                        "{} {mins} min",
                        i18n::t("On duty for:", "En faction depuis :")
                    );
                }
            }
            DaemonStatus::Stopped => {
                println!("{}", i18n::t("State: stopped.", "État : arrêté."));
                println!(
                    "{}",
                    i18n::t(
                        "A `josephine daemon start` and I'm back on guard.",
                        "Un `josephine daemon start` et je reprends la garde.",
                    )
                );
            }
        },
        DaemonAction::Logs => {
            let logs = control.logs(50)?;
            println!("{logs}");
        }
        DaemonAction::Run => {
            josephine_core::daemon::run_daemon_foreground().await?;
        }
    }

    Ok(())
}
