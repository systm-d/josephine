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
}

pub async fn run(action: DaemonAction) -> Result<()> {
    let exe = std::env::current_exe()?;
    let control = DaemonControl::new(exe)?;

    match action {
        DaemonAction::Start => {
            control.start()?;
            println!("✨ Joséphine veille désormais sur votre machine.");
        }
        DaemonAction::Stop => {
            control.stop()?;
            println!("✨ Joséphine s'est endormie. À bientôt.");
        }
        DaemonAction::Restart => {
            control.restart()?;
            println!("✨ Joséphine a repris son poste.");
        }
        DaemonAction::Status => match control.status()? {
            DaemonStatus::Running { pid, started_at } => {
                println!("État : en veille (PID {pid})");
                if let Some(t) = started_at {
                    if let Ok(elapsed) = t.elapsed() {
                        let mins = elapsed.as_secs() / 60;
                        println!("Depuis : {mins} min");
                    }
                }
            }
            DaemonStatus::Stopped => {
                println!("État : au repos");
                println!("Lancez `josephine daemon start` pour activer la surveillance.");
            }
        },
        DaemonAction::Logs => {
            let logs = control.logs(50)?;
            println!("{logs}");
        }
    }

    Ok(())
}
