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
                    "✨ Here I am at my post, eyes open. Go about your day — I'm watching.",
                    "✨ Me voilà à mon poste, l'œil ouvert. Vaquez tranquille, je veille.",
                )
            );
        }
        DaemonAction::Stop => {
            control.stop()?;
            println!(
                "{}",
                i18n::t(
                    "✨ I fold my wings and doze off. Call me at the slightest trouble.",
                    "✨ Je replie mes ailes et m'assoupis. Appelez-moi au moindre souci.",
                )
            );
        }
        DaemonAction::Restart => {
            control.restart()?;
            println!(
                "{}",
                i18n::t(
                    "✨ A flap of the wings and here I am again, fresh and back on watch.",
                    "✨ Un battement d'ailes et me revoilà, fraîche et de nouveau de garde.",
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
                println!(
                    "{}",
                    i18n::t(
                        "State: dozing, wings folded.",
                        "État : assoupie, les ailes repliées."
                    )
                );
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
