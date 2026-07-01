use anyhow::Result;
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
            println!("✨ Configuration impeccable — pas un pli à repasser sur votre petit nuage.");
        }
        ConfigAction::Edit => {
            println!("✨ L'édition guidée patiente encore au vestiaire des anges.");
            println!(
                "   En attendant, ouvrez {} à la main — je relirai par-dessus votre épaule.",
                paths.config.display()
            );
        }
    }

    Ok(())
}
