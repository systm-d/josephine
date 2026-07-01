use std::process::Command;

use anyhow::{Context, Result};
use clap::Subcommand;
use josephine_core::config::Config;
use josephine_core::paths::Paths;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Affiche la configuration actuelle
    Show,
    /// Valide la configuration
    Validate,
    /// Ouvre la configuration dans $EDITOR puis la revalide
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
            println!("✨ Configuration impeccable — pas un pli à repasser sur votre petit nuage.");
        }
        ConfigAction::Edit => {
            // Make sure the file exists (creates a default) before editing.
            Config::load(&paths.config)?;

            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "nano".to_string());

            let status = Command::new(&editor)
                .arg(&paths.config)
                .status()
                .with_context(|| format!("lancement de l'éditeur « {editor} »"))?;

            if !status.success() {
                println!("✨ L'éditeur s'est refermé sans conclure — je n'ai touché à rien.");
                return Ok(());
            }

            match Config::load(&paths.config) {
                Ok(_) => {
                    println!("✨ Configuration relue et validée — pas un pli de travers.")
                }
                Err(e) => {
                    println!("✨ Hmm, votre configuration a un petit accroc :");
                    println!("   {e}");
                    println!(
                        "   Relancez `josephine config edit` pour la remettre d'aplomb — \
                         vos réglages sont conservés tels quels."
                    );
                }
            }
        }
    }

    Ok(())
}
