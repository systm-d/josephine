use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(Clone)]
pub struct Paths {
    pub config: PathBuf,
    pub data_dir: PathBuf,
    pub database: PathBuf,
    pub pid_file: PathBuf,
    pub log_file: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let config = dirs::config_dir()
            .context("impossible de déterminer le répertoire de configuration")?
            .join("josephine")
            .join("config.yaml");

        let data_dir = dirs::data_local_dir()
            .context("impossible de déterminer le répertoire de données")?
            .join("josephine");

        Ok(Self {
            database: data_dir.join("josephine.db"),
            pid_file: data_dir.join("daemon.pid"),
            log_file: data_dir.join("daemon.log"),
            config,
            data_dir,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        if let Some(parent) = self.config.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("création de {}", parent.display()))?;
        }
        std::fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("création de {}", self.data_dir.display()))?;
        Ok(())
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new().expect("chemins système indisponibles")
    }
}
