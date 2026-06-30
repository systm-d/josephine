# Joséphine — Phase 1+2: Standards & Code Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the Joséphine workspace up to the `rust-cli-template` foundation standard (edition 2024 / MSRV 1.85, toolchain, formatting, lints, release profile, dual license, crate rename, template code skeleton, integration tests, CHANGELOG, English README) and adopt the template's SQLite migrations pattern — while keeping the app iso-functional.

**Architecture:** Two crates stay: `josephine-core` (pure logic library) and the binary crate, renamed `josephine-cli` → `josephine`. CLI parsing/dispatch (`cli.rs`, `commands/`, `output/`) stays in the binary crate; `josephine-core` stays pure logic. The binary keeps `#[tokio::main]` but its `main()` becomes a thin delegate to `cli::run() -> ExitCode`, mirroring the template's `run() -> ExitCode` shape while preserving Joséphine's warm French error message. SQLite schema moves from an inline `migrate()` to the template's `MIGRATIONS` + `schema_version` + `apply_migrations()` pattern backed by `migrations/V001__init.sql`.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), clap 4 (derive), anyhow, tokio, rusqlite (bundled), assert_cmd + predicates (integration tests).

## Global Constraints

- **Edition:** `2024` (workspace-wide). **MSRV:** `rust-version = "1.85"`.
- **Formatting:** `rustfmt.toml` with `edition = "2024"`, `max_width = 100`. `cargo fmt --check` must pass.
- **Lints:** `[workspace.lints.rust] unsafe_code = "forbid"`; `[workspace.lints.clippy] all = { level = "warn", priority = -1 }`. Each member crate sets `[lints] workspace = true`. `cargo clippy --workspace --all-targets -- -D warnings` must pass.
- **License:** `MIT OR Apache-2.0`, with both `LICENSE-MIT` and `LICENSE-APACHE` at repo root.
- **Release profile:** `lto = true`, `codegen-units = 1`, `strip = true`.
- **Binary crate name:** `josephine` (directory `crates/josephine/`). Library crate name unchanged: `josephine-core`. The produced binary is `josephine`.
- **Language:** docs (README, CHANGELOG) in **English**; user-facing strings (CLI messages, desktop notifications, `--about` text) stay **French**. The warm error message `✨ Joséphine a rencontré un souci : {e}` is preserved.
- **Iso-functional:** no behavior change to commands; existing v0.1 SQLite databases must keep opening without data loss.
- **Metadata defaults:** repository `https://github.com/systm-d/josephine`, author `Kevin Delfour`, security contact `k@levilainpetit.dev`.
- Reference template files live at `/home/kdelfour/Workspace/Professionel/_templates/cli/`.
- Work happens on branch `chore/align-to-cli-template`.

---

### Task 1: Bump edition to 2024 and set MSRV

**Files:**
- Modify: `Cargo.toml` (workspace root, `[workspace.package]`)

**Interfaces:**
- Produces: workspace compiles under edition 2024 — every later task depends on this.

- [ ] **Step 1: Edit `[workspace.package]`**

In `Cargo.toml`, change `edition = "2021"` to `edition = "2024"` and add `rust-version = "1.85"` immediately after it:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT"
authors = ["Joséphine Contributors"]
repository = "https://github.com/systm-d/josephine"
description = "L'ange gardien de votre ordinateur"
```

(License/authors are fixed in Task 4; leave them for now.)

- [ ] **Step 2: Build to surface edition-migration issues**

Run: `cargo build --workspace`
Expected: PASS. If it fails with edition-2024 migration errors, run `cargo fix --edition --workspace --allow-dirty --allow-staged` then `cargo build --workspace` again until it passes.

- [ ] **Step 3: Run existing tests**

Run: `cargo test --workspace`
Expected: PASS (the 8 existing unit tests).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: migrate workspace to edition 2024, set MSRV 1.85"
```

---

### Task 2: Add toolchain + formatting config and reformat

**Files:**
- Create: `rust-toolchain.toml`
- Create: `rustfmt.toml`
- Modify: all `*.rs` (mechanical reformat via `cargo fmt`)

- [ ] **Step 1: Create `rust-toolchain.toml`**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 2: Create `rustfmt.toml`**

```toml
edition = "2024"
max_width = 100
```

- [ ] **Step 3: Verify formatting is currently non-conformant (expected)**

Run: `cargo fmt --check`
Expected: FAIL (diff reported) — confirms rustfmt is active and will change files.

- [ ] **Step 4: Reformat the workspace**

Run: `cargo fmt`
Then verify: `cargo fmt --check`
Expected: PASS (no diff).

- [ ] **Step 5: Confirm build + tests still pass**

Run: `cargo build --workspace && cargo test --workspace`
Expected: PASS.

- [ ] **Step 6: Commit (reformat isolated for reviewability)**

```bash
git add rust-toolchain.toml rustfmt.toml
git commit -m "build: pin stable toolchain and add rustfmt config"
git add -A
git commit -m "style: apply rustfmt (edition 2024, max_width 100)"
```

---

### Task 3: Add workspace lints and release profile

**Files:**
- Modify: `Cargo.toml` (workspace root — add `[workspace.lints.*]` and `[profile.release]`)
- Modify: `crates/josephine-core/Cargo.toml` (add `[lints] workspace = true`)
- Modify: `crates/josephine-cli/Cargo.toml` (add `[lints] workspace = true`)

**Interfaces:**
- Produces: clippy-clean workspace; `cargo clippy --workspace --all-targets -- -D warnings` passes.

- [ ] **Step 1: Add lints + release profile to root `Cargo.toml`**

Append at the end of `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: Inherit lints in `crates/josephine-core/Cargo.toml`**

Append:

```toml
[lints]
workspace = true
```

- [ ] **Step 3: Inherit lints in `crates/josephine-cli/Cargo.toml`**

Append:

```toml
[lints]
workspace = true
```

- [ ] **Step 4: Run clippy as warnings-as-errors and fix each finding**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: initially may FAIL. Clippy names each lint and its location precisely; apply the suggested fix for each (common ones in this codebase: `clippy::useless_format`, `clippy::needless_return`, `clippy::redundant_clone`, `clippy::manual_map`). Re-run until it PASSES with zero warnings. Do **not** add `#[allow(...)]` unless a lint is genuinely a false positive — note any such case in the commit message.

- [ ] **Step 5: Confirm fmt, build, tests**

Run: `cargo fmt --check && cargo build --workspace && cargo test --workspace`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "build: enforce workspace lints (forbid unsafe, clippy warn) and release profile"
```

---

### Task 4: Dual license (MIT OR Apache-2.0)

**Files:**
- Create: `LICENSE-MIT`
- Create: `LICENSE-APACHE`
- Modify: `Cargo.toml` (`[workspace.package] license`, `authors`)

- [ ] **Step 1: Copy the Apache-2.0 license verbatim from the template**

```bash
cp /home/kdelfour/Workspace/Professionel/_templates/cli/LICENSE-APACHE ./LICENSE-APACHE
```

- [ ] **Step 2: Create `LICENSE-MIT`**

Copy the template's MIT file and set the copyright line to Joséphine's:

```bash
cp /home/kdelfour/Workspace/Professionel/_templates/cli/LICENSE-MIT ./LICENSE-MIT
```

Then edit the copyright line of `LICENSE-MIT` to read:

```
Copyright (c) 2026 Kevin Delfour
```

(Verify the template's placeholder line and replace it; the rest of the MIT text is unchanged.)

- [ ] **Step 3: Update license + authors in root `Cargo.toml`**

In `[workspace.package]`:

```toml
license = "MIT OR Apache-2.0"
authors = ["Kevin Delfour"]
```

- [ ] **Step 4: Verify the manifest is still valid**

Run: `cargo build --workspace`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add LICENSE-MIT LICENSE-APACHE Cargo.toml
git commit -m "chore: dual-license under MIT OR Apache-2.0"
```

---

### Task 5: Rename binary crate `josephine-cli` → `josephine`

**Files:**
- Rename: `crates/josephine-cli/` → `crates/josephine/` (preserve history with `git mv`)
- Modify: `crates/josephine/Cargo.toml` (`package.name`)
- Modify: `Cargo.toml` (workspace `members`)

**Interfaces:**
- Produces: binary crate package name `josephine`; `cargo install --path crates/josephine` yields the `josephine` binary.

- [ ] **Step 1: Move the crate directory (history-preserving)**

```bash
git mv crates/josephine-cli crates/josephine
```

- [ ] **Step 2: Rename the package in `crates/josephine/Cargo.toml`**

Change the package name (the `[[bin]]` block already names the binary `josephine` — leave it):

```toml
[package]
name = "josephine"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
```

(Add `rust-version.workspace = true` if not already present.)

- [ ] **Step 3: Update workspace `members` in root `Cargo.toml`**

```toml
members = ["crates/josephine-core", "crates/josephine"]
```

- [ ] **Step 4: Build, test, and smoke the binary**

Run:
```bash
cargo build --workspace
cargo test --workspace
cargo run -p josephine -- --version
```
Expected: PASS; `--version` prints `josephine 0.1.0`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: rename binary crate josephine-cli -> josephine"
```

---

### Task 6: Complete crate metadata

**Files:**
- Modify: `crates/josephine/Cargo.toml`
- Modify: `crates/josephine-core/Cargo.toml`

- [ ] **Step 1: Add publish metadata to the binary crate `crates/josephine/Cargo.toml` `[package]`**

```toml
[package]
name = "josephine"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
description = "Joséphine — your computer's guardian angel"
documentation = "https://docs.rs/josephine"
readme = "../../README.md"
keywords = ["cli", "monitoring", "system", "guardian", "daemon"]
categories = ["command-line-utilities"]
```

Add `homepage` to root `Cargo.toml` `[workspace.package]` so `homepage.workspace = true` resolves:

```toml
homepage = "https://github.com/systm-d/josephine"
```

- [ ] **Step 2: Align core crate metadata `crates/josephine-core/Cargo.toml` `[package]`**

```toml
[package]
name = "josephine-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Core library for Joséphine — system guardian"
keywords = ["monitoring", "system", "daemon"]
categories = ["os"]
```

- [ ] **Step 3: Verify manifests resolve**

Run: `cargo build --workspace`
Expected: PASS.

- [ ] **Step 4: Verify packaging metadata is well-formed (dry run)**

Run: `cargo package -p josephine --no-verify --allow-dirty --list | head`
Expected: lists files without manifest errors. (Full `cargo package` may warn about uncommitted files — `--allow-dirty` is fine here.)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: complete crate metadata (keywords, categories, homepage, docs)"
```

---

### Task 7: Align code skeleton to template (`run() -> ExitCode` + `cli.rs`)

**Files:**
- Create: `crates/josephine/src/cli.rs`
- Modify: `crates/josephine/src/main.rs`

**Interfaces:**
- Consumes: `commands::{config_cmd, daemon_cmd, doctor_cmd, history_cmd, status_cmd, stub_cmd, ConfigAction, DaemonAction, StubCommand}` (unchanged), `josephine_core::daemon::run_daemon_foreground`.
- Produces: `pub async fn cli::run() -> std::process::ExitCode` — the single entry the binary delegates to.

- [ ] **Step 1: Create `crates/josephine/src/cli.rs`**

Move the `Cli`/`Commands` definitions and dispatch out of `main.rs` into a `cli` module, exposing `run()`:

```rust
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    ConfigAction, DaemonAction, StubCommand, config_cmd, daemon_cmd, doctor_cmd, history_cmd,
    status_cmd, stub_cmd,
};

/// L'ange gardien de votre ordinateur
#[derive(Parser)]
#[command(name = "josephine", about = "L'ange gardien de votre ordinateur", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Mode interne — lancé par `josephine daemon start`
    #[arg(long = "__daemon__", hide = true)]
    daemon_internal: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Résumé rapide de l'état de la machine
    Status,
    /// Diagnostic complet
    Doctor,
    /// Historique des dernières 24 heures
    History,
    /// Gestion du démon de surveillance
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Nettoyage (bientôt)
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
    /// Corrections guidées (bientôt)
    Fix,
    /// Rapport complet (bientôt)
    Report,
}

/// Entry point: parse, dispatch, and map errors to a process exit code.
/// The warm French tone is intentional and preserved.
pub async fn run() -> ExitCode {
    match dispatch().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✨ Joséphine a rencontré un souci : {e}");
            ExitCode::from(1)
        }
    }
}

async fn dispatch() -> Result<()> {
    let cli = Cli::parse();

    if cli.daemon_internal {
        return josephine_core::daemon::run_daemon_foreground()
            .await
            .map_err(Into::into);
    }

    match cli.command {
        Some(Commands::Status) => status_cmd::run()?,
        Some(Commands::Doctor) => doctor_cmd::run()?,
        Some(Commands::History) => history_cmd::run()?,
        Some(Commands::Daemon { action }) => daemon_cmd::run(action).await?,
        Some(Commands::Config { action }) => config_cmd::run(action)?,
        Some(Commands::Clean { dry_run }) => stub_cmd::run(StubCommand::Clean { dry_run })?,
        Some(Commands::Fix) => stub_cmd::run(StubCommand::Fix)?,
        Some(Commands::Report) => stub_cmd::run(StubCommand::Report)?,
        None => status_cmd::run()?,
    }

    Ok(())
}
```

Note: the internal flag field is renamed `__daemon__` → `daemon_internal` (the `#[arg(long = "__daemon__")]` keeps the CLI-facing flag name identical, so behavior is unchanged) to satisfy non-snake-case lints.

- [ ] **Step 2: Replace `crates/josephine/src/main.rs` with the thin delegate**

```rust
mod cli;
mod commands;
mod output;

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    cli::run().await
}
```

- [ ] **Step 3: Build, lint, format**

Run: `cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check`
Expected: PASS.

- [ ] **Step 4: Smoke the dispatch paths**

Run:
```bash
cargo run -p josephine -- --help
cargo run -p josephine -- --version
cargo run -p josephine -- status
```
Expected: help lists the subcommands; version prints `josephine 0.1.0`; `status` renders the status table (same as before).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: extract cli.rs with run() -> ExitCode (template skeleton)"
```

---

### Task 8: CLI integration tests (`assert_cmd` + `predicates`)

**Files:**
- Create: `crates/josephine/tests/cli.rs`
- Modify: `crates/josephine/Cargo.toml` (`[dev-dependencies]`)
- Modify: `Cargo.toml` (workspace deps: `assert_cmd`, `predicates`)

**Interfaces:**
- Consumes: the `josephine` binary (via `assert_cmd::Command::cargo_bin("josephine")`).

- [ ] **Step 1: Add the test dependencies to the workspace `Cargo.toml` `[workspace.dependencies]`**

```toml
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Reference them as dev-dependencies in `crates/josephine/Cargo.toml`**

Append:

```toml
[dev-dependencies]
assert_cmd.workspace = true
predicates.workspace = true
```

- [ ] **Step 3: Write the failing integration test `crates/josephine/tests/cli.rs`**

These tests must avoid touching the real system state where possible; `--version`, `--help`, and the stub commands are pure and safe. (`status` is intentionally **not** asserted here because it reads live hardware.)

```rust
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
```

- [ ] **Step 4: Run to verify the suite builds and passes**

Run: `cargo test -p josephine --test cli`
Expected: PASS (4 tests). If `clean_is_a_friendly_stub` fails because the stub exits non-zero, adjust the assertion to match the stub's actual contract (the stub prints a "bientôt" message and returns `Ok(())`, so `.success()` is correct).

- [ ] **Step 5: Full verification**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "test: add CLI integration tests (assert_cmd + predicates)"
```

---

### Task 9: CHANGELOG (Keep a Changelog)

**Files:**
- Create: `CHANGELOG.md`

- [ ] **Step 1: Create `CHANGELOG.md`**

```markdown
# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Migrated the workspace to Rust edition 2024 (MSRV 1.85).
- Renamed the binary crate `josephine-cli` to `josephine`.
- Dual-licensed under MIT OR Apache-2.0.
- Adopted the shared `rust-cli-template` conventions (toolchain, rustfmt, lints,
  release profile, integration tests).

## [0.1.0] - 2026-06-01

### Added

- Initial release: `status`, `doctor`, `history`, `daemon`, `config` commands.
- Five checks: cpu, memory, disk, temperature, systemd.
- Background daemon with desktop notifications and a 90-day SQLite history.
```

(Use the project's real 0.1.0 date if known; otherwise leave `2026-06-01` and correct it.)

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: add CHANGELOG (Keep a Changelog)"
```

---

### Task 10: English README with badges

**Files:**
- Modify: `README.md` (rewrite in English)
- Create: `docs/README.fr.md` (preserve the current French product README)

- [ ] **Step 1: Preserve the existing French README**

```bash
git mv README.md docs/README.fr.md
```

- [ ] **Step 2: Create the new English `README.md`**

```markdown
# josephine

> Your computer's quiet guardian angel.

[![CI](https://github.com/systm-d/josephine/actions/workflows/ci.yml/badge.svg)](https://github.com/systm-d/josephine/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/josephine.svg)](https://crates.io/crates/josephine)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

Joséphine watches your machine silently and only speaks up when it actually
helps. She monitors CPU, memory, disk, temperature and systemd services,
detects trouble early, and sends warm, plain-language desktop notifications —
never intrusive, always local. No data ever leaves your computer.

> User-facing messages and notifications are intentionally in **French** — that
> is part of Joséphine's character. See [`docs/README.fr.md`](docs/README.fr.md)
> for the French product guide.

## Installation

```sh
cargo install josephine
```

## Usage

```sh
josephine               # quick status summary (default)
josephine status        # CPU, memory, disk, temperature, systemd at a glance
josephine doctor        # detailed diagnostics, check by check
josephine history       # last 24 hours: peaks and notable events
josephine daemon start  # run the background watcher
josephine daemon status # daemon state (PID, uptime)
josephine config show   # print the current configuration
josephine --version
```

Configuration lives at `~/.config/josephine/config.yaml` (created on first run).
History and the daemon's state live under `~/.local/share/josephine/`.

## Development

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
```

- [ ] **Step 3: Sanity-check links and that the FR doc is preserved**

Run: `ls README.md docs/README.fr.md LICENSE-MIT LICENSE-APACHE`
Expected: all four exist.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "docs: English README with badges; preserve French guide under docs/"
```

---

### Task 11: Adopt the migrations pattern for SQLite (Phase 2)

**Files:**
- Create: `crates/josephine-core/migrations/V001__init.sql`
- Modify: `crates/josephine-core/src/storage.rs`

**Interfaces:**
- Consumes: `rusqlite::Connection`.
- Produces: `Storage::open` applies versioned migrations via a `schema_version` table; behavior identical to the prior inline `migrate()` for fresh and existing databases.

- [ ] **Step 1: Create `crates/josephine-core/migrations/V001__init.sql`**

Move the exact schema from the current `storage.rs::migrate()` body into a versioned migration file:

```sql
-- V001: initial schema for Joséphine (metrics, events, notifications, check log).
CREATE TABLE IF NOT EXISTS metrics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name  TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    value       REAL NOT NULL,
    recorded_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_metrics_check_time
    ON metrics(check_name, recorded_at);

CREATE TABLE IF NOT EXISTS events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name  TEXT NOT NULL,
    metric_name TEXT NOT NULL,
    from_state  TEXT NOT NULL,
    to_state    TEXT NOT NULL,
    value       REAL NOT NULL,
    message     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notifications (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    channel  TEXT NOT NULL,
    sent_at  TEXT NOT NULL,
    FOREIGN KEY(event_id) REFERENCES events(id)
);

CREATE TABLE IF NOT EXISTS checks_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    check_name    TEXT NOT NULL,
    status        TEXT NOT NULL,
    duration_ms   INTEGER NOT NULL,
    error_message TEXT,
    ran_at        TEXT NOT NULL
);
```

- [ ] **Step 2: Write the failing migration test in `crates/josephine-core/src/storage.rs`**

Add a `#[cfg(test)] mod tests` at the end of the file (the test references `apply_migrations` and `MIGRATIONS`, which Step 4 will introduce):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_to_in_memory_database() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        let applied: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }

    #[test]
    fn applying_migrations_twice_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        apply_migrations(&conn).unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, MIGRATIONS.len() as i64);
    }
}
```

- [ ] **Step 3: Run the test to verify it fails to compile/pass**

Run: `cargo test -p josephine-core storage::tests`
Expected: FAIL — `apply_migrations` / `MIGRATIONS` not found.

- [ ] **Step 4: Implement the migrations pattern in `crates/josephine-core/src/storage.rs`**

Add the migrations constant and function near the top of the file (after the `use` lines):

```rust
/// Embedded, ordered schema migrations. The version of `MIGRATIONS[i]` is `i + 1`.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/V001__init.sql")];

/// Apply every migration newer than the recorded schema version. Idempotent.
fn apply_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")?;
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    for (index, sql) in MIGRATIONS.iter().enumerate() {
        let version = index as i64 + 1;
        if version > current {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [version])?;
        }
    }
    Ok(())
}
```

Then replace the old `migrate()` usage in `Storage::open` and delete the `fn migrate(&self)` method:

```rust
pub fn open(paths: &Paths) -> Result<Self> {
    paths.ensure_dirs()?;
    let conn = Connection::open(&paths.database)
        .with_context(|| format!("ouverture de {}", paths.database.display()))?;
    apply_migrations(&conn)?;
    Ok(Self { conn })
}
```

(Remove the now-unused `fn migrate(&self) -> Result<()> { ... }` method entirely.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p josephine-core storage::tests`
Expected: PASS (2 tests).

- [ ] **Step 6: Verify backward compatibility with an existing database**

Because every statement in `V001__init.sql` uses `IF NOT EXISTS`, applying it over a v0.1 database is safe: the tables already exist, `schema_version` is created and stamped to `1`. Confirm the full suite and a real open still work:

```bash
cargo test --workspace
cargo run -p josephine -- history
```
Expected: tests PASS; `history` runs without error against any pre-existing `~/.local/share/josephine/josephine.db`.

- [ ] **Step 7: Final verification gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor: adopt versioned SQLite migrations (schema_version + V001__init.sql)"
```

---

## Done criteria for Phase 1+2

- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo build --release --workspace` — succeeds with the LTO/strip profile.
- `cargo test --workspace` — all unit + integration tests pass.
- Binary is `josephine`; `cargo install --path crates/josephine` works.
- Repo has `rust-toolchain.toml`, `rustfmt.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, `CHANGELOG.md`, English `README.md`, `crates/josephine-core/migrations/V001__init.sql`.
- App is iso-functional; existing SQLite databases still open.

## Next plans (separate documents)

- **Phase 3** — Governance & supply-chain (`CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY`, `CONVENTIONS`, `CLAUDE.md`, `.github/` templates, `dependabot`, `CODEOWNERS`, `deny.toml`).
- **Phase 4** — CI (`ci.yml`: lint/test matrix Ubuntu+Fedora/coverage/security/bench-smoke, `tarpaulin.toml`, `benches/`).
- **Phase 5** — Packaging & release, Linux-pragmatic (`[package.metadata.deb]` + `generate-rpm`, `packaging/systemd/josephine.service` user unit, AUR, Homebrew-Linux, `release.yml`).
- **Phase 6** — Zola site + GitHub Pages (`site/`, `pages.yml`, brand color `#E0A458`).
- **Phase 7** — Final verification (CI green, coverage ≥ 80%, release dry-run, Pages deploy).
```