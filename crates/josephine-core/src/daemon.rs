use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, anyhow};
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::Config;
use crate::paths::Paths;
use crate::scheduler::Scheduler;

#[derive(Debug, Clone)]
pub enum DaemonStatus {
    Running {
        pid: u32,
        started_at: Option<SystemTime>,
    },
    Stopped,
}

pub struct DaemonControl {
    paths: Paths,
    exe: std::path::PathBuf,
}

impl DaemonControl {
    pub fn new(exe: std::path::PathBuf) -> Result<Self> {
        let paths = Paths::new()?;
        paths.ensure_dirs()?;
        Ok(Self { paths, exe })
    }

    pub fn status(&self) -> Result<DaemonStatus> {
        match read_pid(&self.paths.pid_file) {
            Some(pid) if process_alive(pid) => {
                let started_at = fs::metadata(&self.paths.pid_file)
                    .ok()
                    .and_then(|m| m.modified().ok());
                Ok(DaemonStatus::Running { pid, started_at })
            }
            Some(_) => {
                let _ = fs::remove_file(&self.paths.pid_file);
                Ok(DaemonStatus::Stopped)
            }
            None => Ok(DaemonStatus::Stopped),
        }
    }

    pub fn start(&self) -> Result<()> {
        match self.status()? {
            DaemonStatus::Running { pid, .. } => {
                return Err(anyhow!(
                    "Joséphine veille déjà sur votre machine (PID {pid})"
                ));
            }
            DaemonStatus::Stopped => {}
        }

        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.paths.log_file)
            .with_context(|| format!("ouverture de {}", self.paths.log_file.display()))?;

        let err_file = log_file.try_clone()?;

        let child = Command::new(&self.exe)
            .arg("--__daemon__")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file))
            .spawn()
            .context("impossible de lancer le démon")?;

        fs::write(&self.paths.pid_file, child.id().to_string())?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        match self.status()? {
            DaemonStatus::Running { pid, .. } => {
                Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status()
                    .context("envoi du signal d'arrêt")?;

                for _ in 0..20 {
                    if !process_alive(pid) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(250));
                }

                let _ = fs::remove_file(&self.paths.pid_file);
                Ok(())
            }
            DaemonStatus::Stopped => Err(anyhow!("Joséphine ne veille pas actuellement")),
        }
    }

    pub fn restart(&self) -> Result<()> {
        let _ = self.stop();
        std::thread::sleep(Duration::from_millis(500));
        self.start()
    }

    pub fn logs(&self, lines: usize) -> Result<String> {
        if !self.paths.log_file.exists() {
            return Ok(String::from("(aucun log pour l'instant)"));
        }
        let content = fs::read_to_string(&self.paths.log_file)?;
        let all_lines: Vec<&str> = content.lines().collect();
        let start = all_lines.len().saturating_sub(lines);
        Ok(all_lines[start..].join("\n"))
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }
}

pub async fn run_daemon_foreground() -> Result<()> {
    let paths = Paths::new()?;
    paths.ensure_dirs()?;

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.log_file)?;

    let subscriber = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("josephine_core=info")),
        )
        .with_writer(log_file)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let pid = std::process::id();
    fs::write(&paths.pid_file, pid.to_string())?;

    let config = Config::load_default()?;
    let mut scheduler = Scheduler::new(config, paths.clone())?;

    let result = scheduler.run().await;

    let _ = fs::remove_file(&paths.pid_file);
    result
}

fn read_pid(path: &Path) -> Option<u32> {
    let mut content = String::new();
    fs::File::open(path)
        .ok()?
        .read_to_string(&mut content)
        .ok()?;
    content.trim().parse().ok()
}

fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}
