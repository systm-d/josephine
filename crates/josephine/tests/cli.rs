use assert_cmd::Command;
use predicates::str::contains;

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
fn daemon_help_lists_run() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(contains("run"));
}
