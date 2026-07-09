//! `josephine clean` — reports reclaimable space (preview by default) and,
//! with `--apply`, clears the always-safe thumbnail cache. Anything requiring
//! root (journals, package caches) is shown as a command, never run silently.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use josephine_core::check::human_size;
use josephine_core::i18n::{self, Lang};

use crate::output::confirm;

pub fn run(apply: bool) -> Result<()> {
    crate::output::sober_header(Some(i18n::t("clean", "clean")), mode_label(apply));

    let cache = cache_dir();
    let thumbnails = cache.join("thumbnails");
    let thumb_size = dir_size(&thumbnails);

    row(
        i18n::t("User cache (~/.cache)", "Cache utilisateur (~/.cache)"),
        dir_size(&cache),
        i18n::t("preview", "aperçu"),
    );
    row(
        i18n::t("  ↳ Thumbnails", "  ↳ Miniatures"),
        thumb_size,
        i18n::t("safe to clear", "nettoyable en sécurité"),
    );
    row(
        i18n::t("Temporary files (/tmp)", "Fichiers temporaires (/tmp)"),
        dir_size(Path::new("/tmp")),
        i18n::t("system-managed", "géré par le système"),
    );
    if let Some(bytes) = journal_usage() {
        row(
            i18n::t("systemd journals", "Journaux systemd"),
            bytes,
            "`journalctl --vacuum-time=7d`",
        );
    }
    println!();

    if !apply {
        println!(
            "{}",
            i18n::t(
                "Preview only — nothing was deleted.",
                "Aperçu seulement — rien n'a été supprimé."
            )
        );
        println!(
            "{}",
            i18n::t(
                "To clear thumbnails: `josephine clean --apply`.",
                "Pour vider les miniatures : `josephine clean --apply`."
            )
        );
        println!(
            "{}",
            i18n::t(
                "Run yourself for the rest:",
                "À lancer vous-même pour le reste :"
            )
        );
        println!(
            "  • {}: sudo journalctl --vacuum-time=7d",
            i18n::t("Journals", "Journaux")
        );
        println!(
            "  • {}: sudo apt clean   (or dnf clean all / pacman -Sc)",
            i18n::t("Packages", "Paquets")
        );
        return Ok(());
    }

    if thumb_size == 0 {
        println!(
            "{}",
            i18n::t(
                "Nothing to clear in thumbnails — already spotless.",
                "Rien à nettoyer du côté des miniatures — c'est déjà tout propre."
            )
        );
        return Ok(());
    }
    let size = human_size(thumb_size as f64);
    let dir = thumbnails.display();
    let question = match i18n::lang() {
        Lang::En => format!("Clear {dir} ({size})?"),
        Lang::Fr => format!("Vider {dir} ({size}) ?"),
    };
    if !confirm(&question)? {
        println!(
            "{}",
            i18n::t(
                "Understood, I'll leave everything in place.",
                "Entendu, je laisse tout en place."
            )
        );
        return Ok(());
    }

    clear_dir_contents(&thumbnails)?;
    println!(
        "{}",
        match i18n::lang() {
            Lang::En =>
                format!("{size} returned to your disk — thumbnails will regenerate on their own."),
            Lang::Fr => format!(
                "{size} rendus à votre disque — les miniatures se régénéreront toutes seules."
            ),
        }
    );
    println!(
        "{}",
        i18n::t(
            "For journals and packages, the commands above are yours to run.",
            "Pour les journaux et paquets, les commandes ci-dessus restent à votre main."
        )
    );
    Ok(())
}

fn mode_label(apply: bool) -> Option<&'static str> {
    if apply {
        None
    } else {
        Some(i18n::t("(preview)", "(aperçu)"))
    }
}

fn row(label: &str, bytes: u64, note: &str) {
    println!("  {label:<30} {:>9}   {note}", human_size(bytes as f64));
}

fn cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache")
}

/// Recursively sum file sizes, never following symlinks.
fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.path().symlink_metadata() else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            total += dir_size(&entry.path());
        } else {
            total += meta.len();
        }
    }
    total
}

fn clear_dir_contents(path: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(path) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let target = entry.path();
        let meta = target
            .symlink_metadata()
            .with_context(|| format!("reading {}", target.display()))?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            let _ = fs::remove_dir_all(&target);
        } else {
            let _ = fs::remove_file(&target);
        }
    }
    Ok(())
}

fn journal_usage() -> Option<u64> {
    let output = Command::new("journalctl")
        .arg("--disk-usage")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_journal_usage(&String::from_utf8_lossy(&output.stdout))
}

/// Pull the size out of `journalctl --disk-usage` ("… take up 1.5G in …").
fn parse_journal_usage(text: &str) -> Option<u64> {
    let marker = "take up ";
    let start = text.find(marker)? + marker.len();
    let token: String = text[start..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .collect();
    parse_human_size(&token)
}

fn parse_human_size(token: &str) -> Option<u64> {
    let token = token.trim();
    let (number, multiplier) = if let Some(n) = token.strip_suffix('G') {
        (n, 1024u64.pow(3))
    } else if let Some(n) = token.strip_suffix('M') {
        (n, 1024u64.pow(2))
    } else if let Some(n) = token.strip_suffix('K') {
        (n, 1024)
    } else if let Some(n) = token.strip_suffix('B') {
        (n, 1)
    } else {
        (token, 1)
    };
    let value: f64 = number.parse().ok()?;
    Some((value * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_human_sizes() {
        assert_eq!(parse_human_size("1.5G"), Some(1_610_612_736));
        assert_eq!(parse_human_size("512M"), Some(536_870_912));
        assert_eq!(parse_human_size("0B"), Some(0));
    }

    #[test]
    fn parses_journalctl_line() {
        let sample = "Archived and active journals take up 1.5G in the file system.\n";
        assert_eq!(parse_journal_usage(sample), Some(1_610_612_736));
    }
}
