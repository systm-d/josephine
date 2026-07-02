use std::process::Command;

use anyhow::{Context, Result};
use clap::Subcommand;
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::paths::Paths;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show the current configuration
    Show,
    /// Validate the configuration
    Validate,
    /// Open the configuration in $EDITOR, then re-validate it
    Edit,
}

pub fn run(action: ConfigAction) -> Result<()> {
    let paths = Paths::new()?;
    paths.ensure_dirs()?;

    match action {
        ConfigAction::Show => {
            let config = Config::load(&paths.config)?;
            println!("{}", serde_yaml::to_string(&config)?);
            println!(
                "{} {}",
                i18n::t("# File:", "# Fichier :"),
                paths.config.display()
            );
        }
        ConfigAction::Validate => {
            let config = Config::load(&paths.config)?;
            config.validate()?;
            println!(
                "{}",
                i18n::t(
                    "✨ Configuration spotless — not a crease to iron on your little cloud.",
                    "✨ Configuration impeccable — pas un pli à repasser sur votre petit nuage.",
                )
            );
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
                .with_context(|| format!("launching editor `{editor}`"))?;

            if !status.success() {
                println!(
                    "{}",
                    i18n::t(
                        "✨ The editor closed without finishing — I touched nothing.",
                        "✨ L'éditeur s'est refermé sans conclure — je n'ai touché à rien.",
                    )
                );
                return Ok(());
            }

            match Config::load(&paths.config) {
                Ok(_) => println!(
                    "{}",
                    i18n::t(
                        "✨ Configuration re-read and validated — not a fold out of place.",
                        "✨ Configuration relue et validée — pas un pli de travers.",
                    )
                ),
                Err(e) => {
                    println!(
                        "{}",
                        i18n::t(
                            "✨ Hmm, your configuration has a little snag:",
                            "✨ Hmm, votre configuration a un petit accroc :",
                        )
                    );
                    println!("   {e}");
                    println!(
                        "{}",
                        i18n::t(
                            "   Run `josephine config edit` again to set it straight — your settings are kept as-is.",
                            "   Relancez `josephine config edit` pour la remettre d'aplomb — vos réglages sont conservés tels quels.",
                        )
                    );
                }
            }
        }
    }

    Ok(())
}
