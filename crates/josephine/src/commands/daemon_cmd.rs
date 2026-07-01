use anyhow::Result;
use clap::Subcommand;
use josephine_core::daemon::{DaemonControl, DaemonStatus};

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Démarre le démon de surveillance
    Start,
    /// Arrête le démon
    Stop,
    /// Redémarre le démon
    Restart,
    /// Affiche l'état du démon
    Status,
    /// Affiche les derniers logs
    Logs,
    /// Exécute le watcher en avant-plan (utilisé par systemd `--user`)
    Run,
}

pub async fn run(action: DaemonAction) -> Result<()> {
    let exe = std::env::current_exe()?;
    let control = DaemonControl::new(exe)?;

    match action {
        DaemonAction::Start => {
            control.start()?;
            println!("✨ Me voilà à mon poste, l'œil ouvert. Vaquez tranquille, je veille.");
        }
        DaemonAction::Stop => {
            control.stop()?;
            println!("✨ Je replie mes ailes et m'assoupis. Appelez-moi au moindre souci.");
        }
        DaemonAction::Restart => {
            control.restart()?;
            println!("✨ Un battement d'ailes et me revoilà, fraîche et de nouveau de garde.");
        }
        DaemonAction::Status => match control.status()? {
            DaemonStatus::Running { pid, started_at } => {
                println!("État : de garde, l'œil ouvert (PID {pid})");
                if let Some(t) = started_at
                    && let Ok(elapsed) = t.elapsed()
                {
                    let mins = elapsed.as_secs() / 60;
                    println!("En faction depuis : {mins} min");
                }
            }
            DaemonStatus::Stopped => {
                println!("État : assoupie, les ailes repliées.");
                println!("Un `josephine daemon start` et je reprends la garde.");
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
