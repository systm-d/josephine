# CLI Increment B — Combler les manques — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Add the four missing capabilities: machine-readable `--json` output, shell completions, terminal-channel notifications, and a French-localised `--help`.

**Architecture:** Extends the released 0.7.1 CLI. `--json` adds `serde::Serialize` to the core check types + a flag path in the display commands. Completions use `clap_complete`. Terminal notifications extend the daemon's alert dispatch. Localised `--help` post-processes the derived `clap::Command` by language before parsing.

**Tech Stack:** Rust (edition 2024), clap 4 (derive), `clap_complete` (new dep), serde/serde_json (already deps), tracing.

## Global Constraints
- Every user-facing **runtime** string ships English AND French via `i18n::t(en, fr)` / `match i18n::lang()`.
- Never `ERROR/FATAL/PANIC/CRASH/ÉCHEC` in user-facing text.
- `--json` output is pure JSON on stdout — no colour, no header, no `✦`, nothing else on stdout.
- Edition 2024, MSRV 1.85, `unsafe_code = "forbid"`. Colour only via `colored`.
- Full gate green: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Branch: `feat/cli-increment-b` (off main @ v0.7.1).

---

### Task 1: `--json` machine-readable output

**Files:** Modify `crates/josephine-core/src/check.rs` (derive Serialize), `crates/josephine/src/cli.rs` (add `--json` flags), `crates/josephine/src/commands/status_cmd.rs`, `doctor_cmd.rs`, `report_cmd.rs`; add a small `crates/josephine/src/output/json.rs`.

Design: a `--json` bool flag on `status`, `doctor`, `report`. When set, the command collects the same `Vec<CheckResult>` it already computes and prints a single JSON document to stdout instead of the rendered view. Severity serialises as `"ok"` / `"warning"` / `"critical"`.

- [ ] **Step 1 (core):** In `check.rs`, add `#[derive(serde::Serialize)]` to `Metric` and `CheckResult`. For `Severity`, add `#[derive(serde::Serialize)] #[serde(rename_all = "lowercase")]` and rename variants in the output to `ok`/`warning`/`critical` via `#[serde(rename = "...")]` on each variant (`Info`→`ok`, `Attention`→`warning`, `Critique`→`critical`). Add a unit test asserting `serde_json::to_string(&Severity::Attention).unwrap() == "\"warning\""`.
- [ ] **Step 2 (json module):** Create `crates/josephine/src/output/json.rs` with:
  ```rust
  use josephine_core::check::{CheckResult, Severity};
  use serde::Serialize;

  #[derive(Serialize)]
  struct JsonCheck<'a> {
      check: &'a str,
      severity: Severity,
      value: Option<&'a str>,   // status_value
      details: &'a [String],
      metrics: &'a [josephine_core::check::Metric],
  }

  /// Print the checks as a single pretty-JSON document to stdout.
  pub fn print_checks(results: &[CheckResult]) {
      let checks: Vec<JsonCheck> = results.iter().map(|r| JsonCheck {
          check: &r.check_name,
          severity: r.worst_severity(),
          value: r.status_value.as_deref(),
          details: &r.details,
          metrics: &r.metrics,
      }).collect();
      println!("{}", serde_json::to_string_pretty(&checks).expect("serialize checks"));
  }
  ```
  Export it from `mod.rs` (`pub use json::print_checks as print_checks_json;`). Add `serde` to `crates/josephine/Cargo.toml` deps (workspace serde with `derive`) and confirm `serde_json` is available there (add if missing).
- [ ] **Step 3 (flags):** In `cli.rs`, add `#[arg(long)] json: bool` to the `Status`, `Doctor`, and `Report` variants; thread it into `status_cmd::run(json)`, `doctor_cmd::run(verbose, json)`, `report_cmd::run(output, json)`.
- [ ] **Step 4 (wire):** In each of the three commands, at the top of `run`, if `json` is true: compute the results (reuse the existing collection path) and call `print_checks_json(&results)`, then return `Ok(())` before any rendered output. For `report --json`, ignore `--output` interaction by simply printing JSON to stdout (document that `--json` implies stdout).
- [ ] **Step 5:** Tests + gate. Add an `assert_cmd` integration test: `josephine status --json` produces parseable JSON (`serde_json::from_slice::<serde_json::Value>` on stdout succeeds and is an array). `cargo test -p josephine`, `clippy -D warnings`, `fmt --check`. Eyeball `cargo run -p josephine -- status --json | jq .`. Commit: `git commit -m "feat(cli): --json machine-readable output for status/doctor/report"`

---

### Task 2: shell completions

**Files:** Modify `crates/josephine/Cargo.toml` (add `clap_complete`), `crates/josephine/src/cli.rs` (new subcommand).

- [ ] **Step 1:** Add `clap_complete = "4"` to `crates/josephine/Cargo.toml` (match the clap major). Add it to the workspace deps in the root `Cargo.toml` if that's the pattern, else pin directly.
- [ ] **Step 2:** Add a `Completions { shell: clap_complete::Shell }` variant to the `Commands` enum (with a `/// Generate shell completions (bash, zsh, fish, …)` doc). In `dispatch`, handle it BEFORE the config-dependent commands:
  ```rust
  Some(Commands::Completions { shell }) => {
      use clap::CommandFactory;
      clap_complete::generate(shell, &mut Cli::command(), "josephine", &mut std::io::stdout());
  }
  ```
- [ ] **Step 3:** Tests + gate. Add an `assert_cmd` test: `josephine completions bash` exits 0 and stdout contains `josephine` (a completion script mentions the binary name). `cargo test -p josephine`, `clippy -D warnings`, `fmt --check`. Eyeball `cargo run -p josephine -- completions zsh | head`. Commit: `git commit -m "feat(cli): josephine completions <shell> via clap_complete"`

---

### Task 3: implement terminal-channel notifications

**Files:** Modify wherever the daemon dispatches an alert notification (find it — likely `crates/josephine-core/src/scheduler.rs` and/or `notify.rs`), reading `crates/josephine-core/src/config.rs` for the `notifications.terminal` field.

Today `notifications.desktop` gates the libnotify call; `notifications.terminal` exists in config but is unimplemented. Implement it as a parallel channel: when an alert message is produced and `config.notifications.terminal` is true, ALSO emit the message through `tracing` (so it lands in the daemon's log / journal / foreground `daemon run`).

- [ ] **Step 1:** Locate the alert-dispatch site (grep `send_josephine`/`send_desktop`/`notifications.desktop` in `josephine-core`). Confirm how `desktop` is gated.
- [ ] **Step 2:** Add the terminal channel next to the desktop one:
  ```rust
  if config.notifications.terminal {
      tracing::warn!(target: "josephine::alert", "{message}");
  }
  if config.notifications.desktop {
      // existing desktop send
  }
  ```
  (Use the actual field/variable names found in step 1. `warn!` is appropriate for alerts; recoveries can use `info!` if the code distinguishes them — match the existing state handling.)
- [ ] **Step 3:** Tests + gate. If there's a testable seam (a function taking the config + message), add a unit test that with `terminal = true` the message is dispatched to the terminal channel (e.g. via a small injectable sink, or assert the function is called — keep it real, not a mock of nothing). If no clean seam exists without refactoring, add a focused test on the gating decision helper instead and note the limitation. `cargo test --workspace`, `clippy -D warnings`, `fmt --check`. Commit: `git commit -m "feat(daemon): implement notifications.terminal (alerts to the log/terminal channel)"`

---

### Task 4: French-localised `--help`

**Files:** Modify `crates/josephine/src/cli.rs`.

clap's derived help is English-only. Localise it by post-processing the derived `Command` when the configured language is French, before parsing.

- [ ] **Step 1:** In `run()`/`dispatch()`, load the config language FIRST (the code already calls `Config::load_default()` then `i18n::set_lang`). Build the command explicitly: `let mut cmd = Cli::command();` (via `clap::CommandFactory`). If `i18n::lang() == Lang::Fr`, mutate the help strings:
  ```rust
  if matches!(i18n::lang(), Lang::Fr) {
      cmd = cmd
          .about("L'ange gardien de votre ordinateur")
          .mut_subcommand("status", |c| c.about("Résumé rapide de la santé de votre machine"))
          .mut_subcommand("doctor", |c| c.about("Diagnostic complet"))
          .mut_subcommand("history", |c| c.about("Les dernières 24 heures"))
          .mut_subcommand("daemon", |c| c.about("Gérer le démon de surveillance"))
          .mut_subcommand("config", |c| c.about("Configuration"))
          .mut_subcommand("clean", |c| c.about("Espace disque récupérable (aperçu par défaut)"))
          .mut_subcommand("fix", |c| c.about("Réparations guidées : ce qui ne va pas et comment y remédier"))
          .mut_subcommand("report", |c| c.about("Rapport système daté, à l'écran ou dans un fichier"))
          .mut_subcommand("notify", |c| c.about("Notifications desktop"))
          .mut_subcommand("update", |c| c.about("Vérifier et installer la dernière version de Joséphine"))
          .mut_subcommand("completions", |c| c.about("Générer les complétions shell (bash, zsh, fish…)"));
  }
  let cli = Cli::from_arg_matches(&cmd.get_matches()).map_err(|e| e.exit())?;
  ```
  (Adjust to the actual clap API in use; `mut_subcommand` takes a closure `FnOnce(Command) -> Command`. Verify the exact signature and that `from_arg_matches` error handling matches the existing `Cli::parse()` behaviour — `clap::Error::exit()` for parse/help/version.)
- [ ] **Step 2:** Because config-language load now must happen before arg parsing, keep the existing daemon-internal fast path working (the `--__daemon__` flag). Make sure `--version`/`--help` still work in English by default (no config or `language: en`) and in French with `language: fr`.
- [ ] **Step 3:** Tests + gate. Add an `assert_cmd` test: with default config (English), `josephine --help` stdout contains an English about ("guardian angel"); this is the safe, always-true assertion. (Testing the French branch requires a config file with `language: fr` in an isolated `$XDG_CONFIG_HOME`/`$HOME` — add it if feasible with a tempdir; otherwise assert the English path and note the French path is manually verified.) Manually verify: create a temp config with `language: fr`, run `--help`, confirm French about lines. `cargo test -p josephine`, `clippy -D warnings`, `fmt --check`. Commit: `git commit -m "feat(cli): French-localised --help via config language"`

---

### Task 5: docs + CHANGELOG + full gate

**Files:** Modify `CHANGELOG.md`, `docs/CURRENT_STATE.md` (note the new capabilities), `docs/ROADMAP.md` (mark increment B delivered).

- [ ] **Step 1:** CHANGELOG `## [Unreleased]`:
  ```markdown
  ### Added

  - **`--json` output** for `status`, `doctor` and `report` — machine-readable
    JSON (no colour/header) for scripting and monitoring.
  - **Shell completions**: `josephine completions <bash|zsh|fish>`.
  - **Terminal notifications**: the daemon now honours `notifications.terminal`,
    emitting alerts to its log/terminal channel alongside the desktop channel.
  - **Localised `--help`**: `--help`/`--version` help text follows the configured
    `language` (English by default, French with `language: fr`).
  ```
- [ ] **Step 2:** In `CURRENT_STATE.md`, add these four to the CLI section; in `ROADMAP.md` mark increment B delivered. Keep both French.
- [ ] **Step 3:** Full gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Eyeball all four features once more. Commit: `git commit -m "docs(changelog): increment B — --json, completions, terminal notifications, localised help"`

---

## Self-Review
- --json → Task 1 ✅ · completions → Task 2 ✅ · terminal notif → Task 3 ✅ · localised help → Task 4 ✅ · docs/changelog → Task 5 ✅.
- Risk: Task 4 (localised help) is the trickiest — clap `mut_subcommand`/`from_arg_matches` wiring must preserve current `--help`/`--version`/error-exit behaviour. Keep the English path byte-identical to today when language is English.
- No placeholders; each task ends with a gate + commit.
