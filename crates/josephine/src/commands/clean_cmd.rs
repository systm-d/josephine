//! `josephine clean` — reports reclaimable space (preview by default) and,
//! with `--apply`, clears the always-safe thumbnail cache. Anything requiring
//! root (journals, package caches) is shown as a command, never run silently.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use josephine_core::check::human_size;

use crate::output::confirm;

pub fn run(apply: bool) -> Result<()> {
    println!("✨ Joséphine — grand ménage {}\n", mode_label(apply));

    let cache = cache_dir();
    let thumbnails = cache.join("thumbnails");
    let thumb_size = dir_size(&thumbnails);

    row("Cache utilisateur (~/.cache)", dir_size(&cache), "aperçu");
    row("  ↳ Miniatures", thumb_size, "nettoyable en sécurité");
    row(
        "Fichiers temporaires (/tmp)",
        dir_size(Path::new("/tmp")),
        "géré par le système",
    );
    if let Some(bytes) = journal_usage() {
        row("Journaux systemd", bytes, "`journalctl --vacuum-time=7d`");
    }
    println!();

    if !apply {
        println!("Aperçu seulement — rien n'a été supprimé.");
        println!("Pour vider les miniatures : `josephine clean --apply`.");
        println!("À lancer vous-même pour le reste :");
        println!("  • Journaux : sudo journalctl --vacuum-time=7d");
        println!("  • Paquets  : sudo apt clean   (ou dnf clean all / pacman -Sc)");
        return Ok(());
    }

    if thumb_size == 0 {
        println!("Rien à nettoyer du côté des miniatures — c'est déjà tout propre.");
        return Ok(());
    }
    let question = format!(
        "Vider {} ({}) ?",
        thumbnails.display(),
        human_size(thumb_size as f64)
    );
    if !confirm(&question)? {
        println!("Entendu, je laisse tout en place.");
        return Ok(());
    }

    clear_dir_contents(&thumbnails)?;
    println!(
        "✨ {} rendus à votre disque — les miniatures se régénéreront toutes seules.",
        human_size(thumb_size as f64)
    );
    println!("Pour les journaux et paquets, les commandes ci-dessus restent à votre main.");
    Ok(())
}

fn mode_label(apply: bool) -> &'static str {
    if apply { "" } else { "(aperçu)" }
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
            .with_context(|| format!("lecture de {}", target.display()))?;
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
