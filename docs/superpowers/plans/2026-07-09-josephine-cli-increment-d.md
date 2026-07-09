# CLI Increment D — Nouvelles capacités — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Add three new machine-protecting checks (filesystem-read-only, time sync, security signals) and a `josephine explain` command that describes every check.

**Architecture:** Each check follows the existing `Check` trait pattern (see `crates/josephine-core/src/checks/kernel.rs` as the reference template). `explain` is a new read-only command with static, bilingual per-check content.

**Tech Stack:** Rust (edition 2024). Reads `/proc/mounts`, `timedatectl`/`chronyc`, `journalctl` — all local.

## Global Constraints
- Every user-facing string ships English AND French via `i18n::t(en, fr)` / `match i18n::lang()`. Never `ERROR/FATAL/PANIC/CRASH/ÉCHEC` in user-facing text.
- 100% local, Linux-only. A check that can't read its source degrades gracefully to an informational "unavailable" (like `kernel`/`smart`), never a false alarm.
- Edition 2024, MSRV 1.85, `unsafe_code = "forbid"`.
- Full gate green: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Branch: `feat/cli-increment-d` (off main @ v0.8.0).

## The "add a check" pattern (6 touchpoints — follow `checks/kernel.rs`)
For a check named `X`:
1. `crates/josephine-core/src/checks/X.rs` — `struct XCheck { config }`, `impl Check` (`name()` → `"X"`, `run()` → `CheckResult`), a pure `build_result(...)` + parsing helpers, unit tests. Degrade to an `unavailable()` `CheckResult` when the source can't be read.
2. `crates/josephine-core/src/checks/mod.rs` — `mod X;`, `pub use X::XCheck;`, a `build_checks` entry (`if config.X.enabled { checks.push(Box::new(XCheck::new(config.X.clone()))) }`), and an `interval_for_check` arm.
3. `crates/josephine-core/src/config.rs` — add a field to `ChecksConfig` (reuse `CheckThresholds` unless the check needs custom fields), wire its `Default`, and any `validate()` bounds. Match how `kernel`/`inode` are configured.
4. `crates/josephine-core/src/messages.rs` — an `alert_message` match arm for `"X"` (bilingual, "chaleur sobre" voice).
5. Display labels: `crates/josephine/src/output/style.rs` `check_label` (`"X" => i18n::t(en, fr)`), `primary_metric` (`"X" => find the metric`); `crates/josephine/src/output/doctor.rs` `metric_label` (the metric's name → label).
6. Register the check in the count everywhere it's stated as "eleven" if that number is user-facing (it is in README/site/docs — leave those to Task 5).

---

### Task 1: `filesystem` check — read-only remount detection

A writable filesystem silently remounted **read-only** is a classic early sign of a failing disk or corruption. Read `/proc/mounts`; for each real, normally-writable mount (skip pseudo/`ro`-by-design like squashfs/iso9660/overlay-lowerdir, `/proc`, `/sys`, `/run`, loop/snap), flag any mounted `ro` that you'd expect to be `rw`. Metric `readonly_mounts` (count); any ≥ 1 is **critical** (data at risk). Degrade to unavailable if `/proc/mounts` is unreadable.

- [ ] Implement `checks/filesystem.rs` (`FilesystemCheck`, name `"filesystem"`) per the pattern, parsing `/proc/mounts` (fields: dev, mount, fstype, options). A mount is flagged when its options contain `ro` AND it's a real read-write-class filesystem (ext4/btrfs/xfs/vfat/f2fs…) not an inherently-ro type. `status_value` e.g. `"all read-write"` / `"1 read-only: « /home »"`. Config: `CheckThresholds` (warning `0.0`? — better: default warning=1, critical=1 so any ro mount is critical; enabled=true, interval 120). Unit tests: a sample `/proc/mounts` with one `ro` ext4 → 1 flagged; all `rw` → 0; squashfs `ro` snap → not flagged.
- [ ] Wire touchpoints 2–5. `messages.rs` alert: EN "« {mount} » is mounted read-only — the filesystem may be failing. Back up and check `dmesg`." / FR equivalent. `check_label`: "Filesystem"/"Système de fichiers". `metric_label` for `readonly_mounts`.
- [ ] Gate + a manual `cargo run -p josephine -- doctor` showing the check. Commit: `feat(checks): filesystem read-only remount detection`

### Task 2: `timesync` check — clock synchronisation

Clock drift / NTP not synchronised breaks logs, TLS validation and cron. Read `timedatectl show` (properties `NTPSynchronized`, `NTP`) — or fall back to `timedatectl status`. Metric `clock_unsynced` (0 = synced, 1 = not synced); not-synced is **attention** (not critical — it's a warning-class issue). Degrade to unavailable if `timedatectl` is absent.

- [ ] Implement `checks/timesync.rs` (`TimesyncCheck`, name `"timesync"`). Parse `timedatectl show --property=NTPSynchronized --property=NTP`. `status_value` e.g. `"in sync"` / `"not synchronised"`. Config `CheckThresholds` (warning=1, critical=2 so unsynced=attention never critical; enabled=true, interval 300). Unit tests on the parser: `NTPSynchronized=yes` → synced; `=no` → unsynced.
- [ ] Wire touchpoints 2–5. `messages.rs` alert: EN "The clock isn't synchronised (NTP). Logs, TLS and cron can drift — `timedatectl set-ntp true` usually fixes it." / FR equivalent. `check_label`: "Clock"/"Horloge". `metric_label` for `clock_unsynced`.
- [ ] Gate + manual doctor. Commit: `feat(checks): timesync (NTP synchronisation) check`

### Task 3: `security` check — recent failed authentications

Repeated failed logins/sudo are worth surfacing. Count recent (last hour) failed-auth lines from `journalctl` (`_SYSTEMD_UNIT=sshd` / PAM "authentication failure" / "Failed password" / "sudo: … authentication failure"). Metric `failed_auths` (count); thresholds warning/critical on the count. Degrade to unavailable if the journal is unreadable (like `kernel`).

- [ ] Implement `checks/security.rs` (`SecurityCheck`, name `"security"`). `journalctl --since "1 hour ago" -o cat -q --no-pager` filtered by patterns (`failed password`, `authentication failure`, `invalid user`). `status_value` e.g. `"no failed logins (1 h)"` / `"7 failed logins (1 h)"`. Config `CheckThresholds` (warning=5, critical=20, enabled=true, interval 300). Unit tests on the counter with a sample journal.
- [ ] Wire touchpoints 2–5. `messages.rs` alert: EN "{n} failed login attempts in the last hour. If that's not you, check `journalctl -u sshd`." / FR equivalent. `check_label`: "Security"/"Sécurité". `metric_label` for `failed_auths`.
- [ ] Gate + manual doctor. Commit: `feat(checks): security signals (recent failed authentications)`

### Task 4: `josephine explain` command

A new command that explains each check: what it measures, its thresholds, why it matters, and how to remedy it — bilingual, sober. `josephine explain` lists all checks with a one-line summary; `josephine explain <check>` prints the full explanation for one.

- [ ] Add `Explain { check: Option<String> }` to `cli.rs` `Commands` (doc: "Explain what each check watches, and how to act"), dispatch to a new `crates/josephine/src/commands/explain_cmd.rs`. Use the sober header (`sober_header(Some("explain"), None)`).
- [ ] `explain_cmd`: a table of (check_name, what, why, remedy) for ALL 14 checks (the 11 existing + filesystem/timesync/security), each field bilingual via `i18n::t`. `josephine explain` → the list (name + one-line "what"); `josephine explain cpu` → the full block for that check; unknown name → a friendly "unknown check, here are the names" message (exit 0). Reuse `style::check_label`/`status_glyph` for consistency.
- [ ] Integration test (`assert_cmd`): `josephine explain` exits 0 and lists `cpu`/`filesystem`; `josephine explain disk` mentions disk. Gate. Commit: `feat(cli): josephine explain — what each check watches and how to act`

### Task 5: docs, counts, CHANGELOG, gate

- [ ] Update the check count everywhere it's user-facing: now **fourteen** checks. `README.md`, `site/content/_index{,.fr}.md` (the "eleven checks" lines and lists), `docs/CURRENT_STATE.md` (checks table + count), `docs/ROADMAP.md` (mark increment D delivered). Keep site/docs languages as they are.
- [ ] CHANGELOG `## [Unreleased]` `### Added`: the three new checks (filesystem read-only, timesync, security) and `josephine explain`.
- [ ] Full gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, plus `cargo run -p josephine -- status` (14 checks) and `josephine explain`. Commit: `docs: fourteen checks + explain (increment D)`

## Self-Review
- 3 checks → Tasks 1–3, each self-contained via the 6-touchpoint pattern. explain → Task 4. docs/counts/changelog → Task 5.
- Risk: the "eleven checks" count is stated in several user-facing places (README, site, docs) — Task 5 must catch them all; grep `eleven|onze|11 check|11 contr` to be sure.
- Each check degrades gracefully (no false alarms) and ships EN+FR copy.
