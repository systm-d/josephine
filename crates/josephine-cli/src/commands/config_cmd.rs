use anyhow::{Result, bail};
use clap::Subcommand;
use josephine_core::config::Config;
use josephine_core::paths::Paths;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Affiche la configuration actuelle
    Show,
    /// Valide la configuration
    Validate,
    /// Édition interactive (bientôt)
    Edit,
}

pub fn run(action: ConfigAction) -> Result<()> {
    let paths = Paths::new()?;
    paths.ensure_dirs()?;

    match action {
        ConfigAction::Show => {
            let config = Config::load(&paths.config)?;
            println!("{}", serde_yaml::to_string(&config)?);
            println!("# Fichier : {}", paths.config.display());
        }
        ConfigAction::Validate => {
            let config = Config::load(&paths.config)?;
            config.validate()?;
            println!("✨ Configuration valide.");
        }
        ConfigAction::Edit => {
            bail!(
                "Cette fonctionnalité arrive bientôt avec Joséphine.\nÉditez {} manuellement.",
                paths.config.display()
            );
        }
    }

    Ok(())
}
