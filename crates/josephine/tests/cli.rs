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
fn clean_is_a_friendly_stub() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["clean", "--dry-run"])
        .assert()
        .success();
}

#[test]
fn unknown_command_fails() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("definitely-not-a-command")
        .assert()
        .failure();
}
