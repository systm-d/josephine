//! `josephine update` — check GitHub Releases and, with the user's blessing,
//! download and install the package matching how Joséphine was installed.
//!
//! This is the ONLY place Joséphine touches the network, and only when the user
//! runs the command explicitly — never in the background (the "100 % local" rule).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Duration;

use anyhow::{Context, Result};
use josephine_core::messages;
use josephine_core::paths::Paths;
use josephine_core::update::{self, Asset, InstallPlan, ReleaseInfo, UpdateStatus};

use crate::output::{is_tty, print_banner};

const USER_AGENT: &str = concat!("josephine/", env!("CARGO_PKG_VERSION"));

/// Outcome of comparing the downloaded package against its published checksum.
enum Integrity {
    /// Checksum matched.
    Verified,
    /// No checksum published, or it couldn't be fetched/computed.
    Unverified,
    /// Checksum was published and did NOT match — do not install.
    Mismatch,
}

pub fn run(check_only: bool, assume_yes: bool) -> Result<()> {
    print_banner("Mise à jour");

    let release = match fetch_latest() {
        Ok(release) => release,
        Err(e) => {
            // A network hiccup shouldn't read like a crash.
            println!(
                "Je n'ai pas pu joindre GitHub pour regarder les nouveautés. \
                 On réessaiera plus tard, sans stress."
            );
            println!("(détail : {e})");
            return Ok(());
        }
    };

    let current = update::current_version();
    match update::compare(current, &release.version) {
        UpdateStatus::UpToDate => println!("{}", messages::update_up_to_date(current)),
        UpdateStatus::Ahead => println!("{}", messages::update_ahead(current, &release.version)),
        UpdateStatus::Available(version) => {
            println!("{}", messages::update_available(&version));
            if !release.html_url.is_empty() {
                println!("Notes de version : {}", release.html_url);
            }
            if check_only {
                println!("Quand vous voulez : `josephine update` pour l'installer.");
            } else {
                apply(&release, assume_yes)?;
            }
        }
    }
    Ok(())
}

fn fetch_latest() -> Result<ReleaseInfo> {
    let body = ureq::get(update::LATEST_RELEASE_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(15))
        .call()
        .context("requête vers l'API GitHub")?
        .into_string()
        .context("lecture de la réponse GitHub")?;
    update::parse_release(&body)
}

fn apply(release: &ReleaseInfo, assume_yes: bool) -> Result<()> {
    let channel = update::detect_channel();

    // Channels we can't drive from a downloaded package: just show the command.
    let Some(asset) = release.asset_for(channel) else {
        match update::install_plan(channel, Path::new("")) {
            InstallPlan::Manual(message) => println!("{message}"),
            InstallPlan::Run { .. } => {
                println!("Récupérez la dernière version : {}", release.html_url);
            }
        }
        return Ok(());
    };

    if !assume_yes && !confirm(&format!("Télécharger et installer « {} » ?", asset.name))? {
        println!("Entendu, ce sera pour une autre fois. Je reste de garde.");
        return Ok(());
    }

    let dir = Paths::new()?.data_dir.join("updates");
    println!("Téléchargement de {} …", asset.name);
    let package = download(asset, &dir)?;

    match verify(release, asset, &package) {
        Integrity::Mismatch => {
            println!(
                "⚠ L'empreinte du paquet ne correspond pas à celle publiée. \
                 Par prudence, je n'installe rien — vérifiez votre connexion et réessayez."
            );
            return Ok(());
        }
        Integrity::Verified => println!("Intégrité vérifiée ✓"),
        Integrity::Unverified => {}
    }

    match update::install_plan(channel, &package) {
        InstallPlan::Run { command, sudo } => {
            println!("J'installe la nouvelle version — votre mot de passe peut être demandé.");
            let status = run_install(&command, sudo)?;
            if status.success() {
                println!("{}", messages::update_done(&release.version));
            } else {
                println!(
                    "L'installation ne s'est pas terminée. Le paquet reste prêt ici : {}",
                    package.display()
                );
            }
        }
        InstallPlan::Manual(message) => println!("{message}"),
    }
    Ok(())
}

fn download(asset: &Asset, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).with_context(|| format!("création de {}", dir.display()))?;
    let dest = dir.join(&asset.name);

    let response = ureq::get(&asset.download_url)
        .set("User-Agent", USER_AGENT)
        .call()
        .context("téléchargement du paquet")?;
    let mut reader = response.into_reader();
    let mut file =
        std::fs::File::create(&dest).with_context(|| format!("création de {}", dest.display()))?;
    io::copy(&mut reader, &mut file).context("écriture du paquet téléchargé")?;
    Ok(dest)
}

/// Verify the package against its published `.sha256`, if one exists. A missing
/// or unreachable checksum isn't fatal — we fall back to HTTPS trust.
fn verify(release: &ReleaseInfo, asset: &Asset, package: &Path) -> Integrity {
    let Some(sum_asset) = release.checksum_for(asset) else {
        return Integrity::Unverified;
    };
    let expected = ureq::get(&sum_asset.download_url)
        .set("User-Agent", USER_AGENT)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .and_then(|text| update::parse_sha256_line(&text));
    let actual = update::sha256_hex(package).ok();

    match (expected, actual) {
        (Some(e), Some(a)) if e == a => Integrity::Verified,
        (Some(_), Some(_)) => Integrity::Mismatch,
        _ => Integrity::Unverified,
    }
}

fn confirm(question: &str) -> Result<bool> {
    // Non-interactive shells never auto-install without an explicit `--yes`.
    if !is_tty() {
        return Ok(false);
    }
    print!("{question} [o/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_lowercase().as_str(),
        "o" | "oui" | "y" | "yes"
    ))
}

fn run_install(command: &[String], sudo: bool) -> Result<ExitStatus> {
    let mut argv: Vec<&str> = Vec::with_capacity(command.len() + 1);
    if sudo {
        argv.push("sudo");
    }
    argv.extend(command.iter().map(String::as_str));

    let (program, args) = argv
        .split_first()
        .expect("commande d'installation non vide");
    // Inherit stdio so the user can type their sudo password.
    Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("lancement de « {} »", argv.join(" ")))
}
