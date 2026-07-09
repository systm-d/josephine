# CLI Increment C — Solidité & finitions — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Finish the "Constellation sobre" work: bring the stale docs to 0.7.0, sweep the remaining commands onto the sober header + detoned voice, unify the surfaces' bottom-edge, and close the known copy/test nits.

**Architecture:** Extends increment A (already released in v0.7.0). Consumes the existing `crate::output::sober_header` and the `style.rs` primitives. Pure finishing work — no new subsystems.

**Tech Stack:** Rust (edition 2024), `colored`, `comfy-table`. Prior design: `docs/superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md`. Follow-ups came from that increment's whole-branch review.

## Global Constraints
- Every user-facing **runtime** string ships English AND French via `i18n::t(en, fr)` / `match i18n::lang()`. (Contributor docs `CURRENT_STATE.md`/`ROADMAP.md` are French project docs — keep them French; they are not runtime strings.)
- Never `ERROR/FATAL/PANIC/CRASH/ÉCHEC` in user-facing text.
- Colour only via `colored`/comfy-table API — never raw ANSI in cell text.
- The visual language is fixed: sober `✦` header via `sober_header`, status dots `●▲✕` (degrade to uniform `[ok]/[!] /[x] ` off-TTY), no emoji/`♥`/rounded box, `banner.txt` still honoured.
- Edition 2024, MSRV 1.85, `unsafe_code = "forbid"`.
- Full gate green: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Branch: `feat/cli-increment-c` (off main @ v0.7.0).

---

### Task 1: Refresh the state & roadmap docs to 0.7.0

**Files:** Modify `docs/CURRENT_STATE.md`, `docs/ROADMAP.md`.

These are stuck at 0.3.0. Read the real current state from code before editing: workspace version in `Cargo.toml` (0.7.0), the checks in `crates/josephine-core/src/checks/` (11: cpu, memory, disk, temperature, systemd, updates, network, battery, inode, smart, kernel), the CLI commands in `crates/josephine/src/cli.rs` (status, doctor `-v`, history, daemon, config, clean, fix, report, notify, update), and the i18n/`language` config (`crates/josephine-core/src/config.rs`, English default since 0.5.0).

- [ ] **Step 1:** In `CURRENT_STATE.md`: bump the version header to **0.7.0**; correct the inode source row from `df -iP` to `df -iPT`; ensure the checks table lists all 11 and the CLI table lists all commands; add a short note that `status`/`doctor`/`history` render in the sober "Constellation sobre" style (status dots, `✦` header, detoned bilingual copy) as of 0.7.0; fix any other statements now false at 0.7.0 (e.g. language default). Keep the document's French, structure, and tone.
- [ ] **Step 2:** In `ROADMAP.md`: update the "Baseline actuelle" line to **v0.7.0**; mark the v0.5 "Prévoyance" checks as delivered if not already; add a short entry reflecting the CLI redesign (increment A) as delivered and the increment program (A done, C in progress, B/D planned). Keep it concise and French.
- [ ] **Step 3:** Verify no code claim is wrong: re-read your edits against `cli.rs` and `checks/`. Then commit:
  `git commit -m "docs: refresh CURRENT_STATE & ROADMAP to the 0.7.0 baseline"`

---

### Task 2: Full sweep — sober header + detoned voice across ALL remaining commands

**Files:** Modify `crates/josephine/src/commands/fix_cmd.rs`, `clean_cmd.rs`, `report_cmd.rs`, `update_cmd.rs`, `daemon_cmd.rs`, `config_cmd.rs`, `notify_cmd.rs`; and `crates/josephine/src/output/style.rs` + `mod.rs` if `print_banner` becomes unused.

The redesign (increment A) detoned `status`/`doctor`/`history` + notifications, but the other commands still open with the old `✨ Joséphine` banner / titles and carry mascot copy (`✨`, wings/"ailes", "petit nuage", "carnet de bord tout frais", "file un parfait bonheur", "battement d'ailes"). Sweep them all so the whole CLI reads as one system.

Known cutesy sites to detone (grep `✨` and re-verify — there may be a couple more):
- `fix_cmd.rs:17-18` title `"✨ Joséphine — guided fixes"`; `:85-86` "…parfait bonheur ✨"; `:93-94` "…you keep the wheel ✨".
- `clean_cmd.rs:18` title `"✨ Joséphine — big clean-up"`; `:115-118` "✨ {size} returned…".
- `report_cmd.rs:30-32` "✨ Report saved … a fresh logbook / carnet de bord tout frais".
- `update_cmd.rs:18,30` uses `print_banner(i18n::t("Update","Mise à jour"))`.
- `daemon_cmd.rs:32-53` start/stop/restart messages ("✨ Here I am at my post…", "flap of the wings", "battement d'ailes").
- `config_cmd.rs:39-81` "✨ Configuration spotless… your little cloud / petit nuage", "not a crease to iron", "petit accroc".
- `notify_cmd.rs:19-26` "my wings reach your desktop / mes ailes touchent…", "✨ Notification sent".

- [ ] **Step 1 (headers):** In the commands that open with a banner/title — `fix`, `clean`, `report`, `update` — replace the hard-coded `"✨ Joséphine — …"` title or the `print_banner(subtitle)` call with `crate::output::sober_header(Some(<short suffix>), None)` (suffix e.g. `fix` / `clean` / `report` / `update`), matching `status`/`doctor`/`history`. (`daemon`/`config`/`notify` don't print a header banner — they only need copy detoning.)
- [ ] **Step 2 (copy):** Detone every `✨`/mascot string across all seven command files to the "chaleur sobre" voice — direct, calm, no `✨`/`♥`/wings/clouds/logbooks — keeping each string bilingual EN+FR via `i18n::t`. Preserve all interpolations (`{size}`, `{done}`, paths, counts) and control flow; change only wording. Do NOT touch `cli.rs`'s deliberately-kept clap `about` ("Your computer's guardian angel") or the `cli.rs` error prefix.
- [ ] **Step 3:** After the header migration, check `git grep 'print_banner' crates/josephine/src`. If `print_banner` has no caller left, remove it from `style.rs` and its `mod.rs` re-export (let clippy `-D warnings` confirm). If a caller remains, leave it.
- [ ] **Step 4:** Run `cargo test -p josephine`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (all clean); grep `✨` across `crates/josephine/src` and confirm zero remain (except any deliberately kept — there should be none in command output). Eyeball `report` / `clean` / `fix` / `update --check` / `config show` / `notify test` — sober `✦` header where applicable, no `✨`, detoned copy. Note observations. Commit:
  `git commit -m "feat(cli): sober header + detoned voice across all remaining commands"`

---

### Task 3: Unify the surfaces' bottom edge + label the history events

**Files:** Modify `crates/josephine/src/output/style.rs` (add a shared footer helper), `crates/josephine/src/output/status.rs`, `crates/josephine/src/output/doctor.rs`, `crates/josephine/src/commands/history_cmd.rs`.

Today the bottom edge differs: `status` closes with a `─` rule + a one-space-indented dimmed footer; `doctor` uses `print_footer` (no rule, no indent); `history` has neither.

- [ ] **Step 1:** Add `pub fn sober_footer(msg: &str)` to `style.rs`: prints `"─".repeat(HEADER_WIDTH)` dimmed, then the message as `" {msg}"` dimmed on TTY / plain otherwise. Export it from `mod.rs`.
- [ ] **Step 2:** Route `status`'s closing rule+footer and `doctor`'s footer through `sober_footer` so both share the exact same bottom treatment. If `print_footer` becomes unused afterward, remove it (let clippy decide).
- [ ] **Step 3:** In `history_cmd.rs`, render the **events** table's check column through `check_label(&event.check_name)` (so it reads "Réseau", not "network"), matching the trend table. (Leave the transition `from → to` text as-is; a status glyph on events is deferred — it needs a state→severity mapping, out of this increment.)
- [ ] **Step 4:** Run the gate clean and eyeball `status`/`doctor`/`history` (TTY + `| cat`) — confirm the three now share the same bottom rule/footer convention and history events show human labels. Commit:
  `git commit -m "feat(output): shared sober footer across surfaces; label history events"`

---

### Task 4: Close the known copy/test nits + CHANGELOG + gate

**Files:** Modify `crates/josephine/src/output/status.rs` (test), `crates/josephine-core/src/messages.rs`, `CHANGELOG.md`.

- [ ] **Step 1:** In `status.rs`'s `footer_message_pluralizes` test, stop leaving global language state dirty: capture the current lang, `set_lang(En)`, run the asserts, then restore the captured lang (so a future `Fr`-setting test can't be affected). Keep the same assertions.
- [ ] **Step 2:** In `messages.rs`, align the three EN/FR lexical drifts flagged in review: recovery "network" (EN "steady" vs FR "fluide" → make them equivalent, e.g. EN "stable and steady" / FR "stable et constant"); recovery "battery" (EN "healthy level" vs FR "niveau correct" → align register, e.g. FR "niveau sain"); `update_done` FR "en version {v}" → smoother (e.g. "est maintenant sur la version {v}" or "passe en {v}"). Keep all bilingual, keep the messages tests green (`cargo test -p josephine-core messages`).
- [ ] **Step 3:** Add a `## [Unreleased]` CHANGELOG entry summarising increment C:
  ```markdown
  ### Changed

  - Swept the remaining commands (`report`, `clean`, `fix`, `update`) onto the
    sober `✦` header and the direct "chaleur sobre" voice, and unified the bottom
    rule/footer across `status`/`doctor`/`history`, so the whole CLI now reads as
    one system. `history` events show human check labels. Docs refreshed to 0.7.0.
  ```
- [ ] **Step 4:** Full gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Then commit:
  `git commit -m "fix(cli): tidy EN/FR copy, guard lang test; changelog for increment C"`

---

## Self-Review
- Docs → Task 1 ✅ · copy sweep → Task 2 ✅ · coherence (footer + events label) → Task 3 ✅ · nits + changelog → Task 4 ✅.
- No placeholders; each task ends with a gate + commit.
- Deferred (noted, not this increment): status-glyph on history events (needs state→severity mapping); the value-tint difference between status (tints values) and doctor (tints only state word) — left as a defensible, intentional distinction; `notifications.terminal` dead config → belongs to increment B (implement the terminal channel) not a bare removal.
