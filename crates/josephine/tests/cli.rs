use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

/// A throwaway `HOME` so config/DB-touching commands never read or write the
/// developer's real files (and stay deterministic in CI).
fn isolated_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("josephine-it-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn prints_version() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("josephine"));
}

#[test]
fn help_lists_core_subcommands() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("status"))
        .stdout(contains("doctor"))
        .stdout(contains("daemon"));
}

#[test]
fn clean_previews_without_deleting() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("clean")
        .assert()
        .success()
        .stdout(contains("aperçu"));
}

#[test]
fn help_lists_new_subcommands() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("report"))
        .stdout(contains("update"))
        .stdout(contains("notify"));
}

#[test]
fn unknown_command_fails() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("definitely-not-a-command")
        .assert()
        .failure();
}

#[test]
fn history_runs() {
    // Reads config + SQLite only (no system checks), so it's CI-safe.
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("history"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .arg("history")
        .assert()
        .success();
}

#[test]
fn config_validate_accepts_the_default() {
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("config"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .args(["config", "validate"])
        .assert()
        .success()
        .stdout(contains("Configuration"));
}

#[test]
fn daemon_help_lists_run() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(contains("run"));
}
