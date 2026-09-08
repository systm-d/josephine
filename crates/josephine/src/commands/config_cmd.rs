use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::paths::Paths;
use josephine_core::profile::Profile;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show the current configuration
    Show,
    /// Validate the configuration
    Validate,
    /// Open the configuration in $EDITOR, then re-validate it
    Edit,
    /// Write a starter configuration for a kind of machine
    Init {
        /// laptop, desktop or server
        #[arg(long, value_name = "NAME", default_value = "laptop")]
        profile: String,
        /// Replace an existing configuration
        #[arg(long)]
        force: bool,
    },
}

pub fn run(action: ConfigAction) -> Result<()> {
    let paths = Paths::new()?;
    paths.ensure_dirs()?;

    match action {
        ConfigAction::Init { profile, force } => {
            // Read the state of play before anything creates a file. An
            // existing config also decides which language to answer in.
            let existing = paths.config.exists();
            if existing && let Ok(current) = Config::load(&paths.config) {
                i18n::set_lang(current.language);
            }

            let profile: Profile = profile.parse().map_err(anyhow::Error::msg)?;

            if existing && !force {
                bail!(match i18n::lang() {
                    i18n::Lang::En => format!(
                        "You already have a configuration at {}. \
                         I won't overwrite it — pass --force if that's what you want.",
                        paths.config.display()
                    ),
                    i18n::Lang::Fr => format!(
                        "Vous avez déjà une configuration dans {}. \
                         Je ne l'écrase pas — passez --force si c'est ce que vous voulez.",
                        paths.config.display()
                    ),
                });
            }

            let config = profile.config();
            // A preset that cannot be loaded back would be a config file she
            // then refuses to start with.
            config.validate()?;

            let header = match i18n::lang() {
                i18n::Lang::En => format!(
                    "# Joséphine — starter configuration for a {profile}.\n\
                     # {}\n\
                     # Yours now: edit freely, nothing reads the profile name again.\n",
                    profile.description()
                ),
                i18n::Lang::Fr => format!(
                    "# Joséphine — configuration de départ pour un poste « {profile} ».\n\
                     # {}\n\
                     # Elle est à vous : modifiez-la librement, le nom du profil n'est plus relu.\n",
                    profile.description()
                ),
            };
            let body = serde_yaml::to_string(&config)?;
            std::fs::write(&paths.config, format!("{header}{body}"))
                .with_context(|| format!("writing {}", paths.config.display()))?;

            println!(
                "{}",
                match i18n::lang() {
                    i18n::Lang::En => format!(
                        "Written a {profile} configuration to {}.",
                        paths.config.display()
                    ),
                    i18n::Lang::Fr => format!(
                        "Configuration « {profile} » écrite dans {}.",
                        paths.config.display()
                    ),
                }
            );
            println!("   {}", profile.description());
        }
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
                    "Configuration valid — nothing to fix.",
                    "Configuration valide — rien à corriger.",
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
                        "The editor closed without finishing — I touched nothing.",
                        "L'éditeur s'est refermé sans conclure — je n'ai touché à rien.",
                    )
                );
                return Ok(());
            }

            match Config::load(&paths.config) {
                Ok(_) => println!(
                    "{}",
                    i18n::t(
                        "Configuration re-read and validated.",
                        "Configuration relue et validée.",
                    )
                ),
                Err(e) => {
                    println!(
                        "{}",
                        i18n::t(
                            "Your configuration has an issue:",
                            "Votre configuration a un problème :",
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
