# CLI Render & Tone ("Constellation sobre") Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `status`, `doctor`, `history` and the desktop notifications a sober, grown-up look ("Constellation sobre") and a direct "chaleur sobre" voice, dropping the cutesy banner/emoji/hearts while keeping a discreet stellar identity.

**Architecture:** A small set of shared visual primitives (status glyph, palette, sober header/footer) lands in `crates/josephine/src/output/style.rs` first; the three display surfaces are then rewritten to consume them. Copy is detoned in place, always as EN+FR pairs. `messages.rs` (desktop notifications) gets the same tone pass.

**Tech Stack:** Rust (edition 2024), `colored` (auto-strips styling on non-TTY), `comfy-table`, `chrono`. Design spec: `docs/superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md`.

## Global Constraints

- Edition 2024, MSRV 1.85. `unsafe_code = "forbid"`.
- Every user-facing string ships **English and French** via `i18n::t(en, fr)` or `match i18n::lang()`. No English-only literals.
- Never `ERROR` / `FATAL` / `PANIC` / `CRASH` / `ÉCHEC` in user-facing text (enforced by existing `messages.rs` tests).
- Status severity is carried by **shape *and* colour** so it survives colour-blindness and non-TTY: `●`/`▲`/`✕` on a TTY, `[ok]`/`[!]`/`[x]` when stdout is not a terminal.
- Colour comes from `colored` (which strips itself on non-TTY) or the comfy-table style API — **never raw ANSI inside comfy-table cell text**.
- A custom `~/.config/josephine/banner.txt`, when present and non-empty, is still printed above the header.
- Linux-only. Quality gate must pass:
  `cargo fmt --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo run -p josephine -- status`.
- Work happens on branch `feat/cli-render-tone` (already created off `main`).

---

### Task 1: Shared visual primitives in `style.rs`

Adds the "Constellation sobre" building blocks used by every surface. This is the only task with pure-logic unit tests; the rest verify by integration/manual.

**Files:**
- Modify: `crates/josephine/src/output/style.rs`
- Modify: `crates/josephine/src/output/mod.rs` (exports)

**Interfaces:**
- Produces:
  - `pub fn status_glyph(severity: Severity) -> String` — `●/▲/✕` (coloured) on a TTY, `[ok]/[!]/[x]` off-TTY.
  - `pub fn severity_paint(s: &str, severity: Severity) -> String` — green/amber/red via `colored`.
  - `pub fn accent(s: &str) -> String` — soft indigo/violet truecolor (stellar accent).
  - `pub fn sober_header(suffix: Option<&str>, tagline: Option<&str>)` — prints `✦ Joséphine[ · suffix]` + right-aligned clock, optional dimmed tagline, thin rule; honours `banner.txt`.
  - `pub const HEADER_WIDTH: usize = 54;`
- Consumes: `Severity` from `josephine_core::check`.

- [ ] **Step 1: Write the failing test** (append to the `#[cfg(test)] mod tests` in `style.rs`)

```rust
#[test]
fn status_glyph_is_ascii_off_tty() {
    // `cargo test` stdout is not a terminal, so we get the plain fallback —
    // and it must differ per severity by shape, not colour alone.
    use josephine_core::check::Severity;
    assert_eq!(status_glyph(Severity::Info), "[ok]");
    assert_eq!(status_glyph(Severity::Attention), "[!]");
    assert_eq!(status_glyph(Severity::Critique), "[x]");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p josephine --lib status_glyph_is_ascii_off_tty`
Expected: FAIL — `cannot find function status_glyph`.

- [ ] **Step 3: Add the implementation** to `style.rs`

```rust
use colored::Color;
use josephine_core::check::Severity;

/// Width of the header rule and the clock's right edge.
pub const HEADER_WIDTH: usize = 54;

/// Soft indigo/violet — Joséphine's discreet stellar accent.
const ACCENT: (u8, u8, u8) = (150, 130, 220);

pub fn accent(s: &str) -> String {
    s.truecolor(ACCENT.0, ACCENT.1, ACCENT.2).to_string()
}

fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Info => Color::Green,
        Severity::Attention => Color::Yellow,
        Severity::Critique => Color::Red,
    }
}

pub fn severity_paint(s: &str, severity: Severity) -> String {
    s.color(severity_color(severity)).to_string()
}

/// Status glyph carrying severity by shape *and* colour. Off a terminal it
/// degrades to an ASCII tag so pipes and logs stay readable.
pub fn status_glyph(severity: Severity) -> String {
    let (glyph, plain) = match severity {
        Severity::Info => ("●", "[ok]"),
        Severity::Attention => ("▲", "[!]"),
        Severity::Critique => ("✕", "[x]"),
    };
    if is_tty() {
        severity_paint(glyph, severity)
    } else {
        plain.to_string()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p josephine --lib status_glyph_is_ascii_off_tty`
Expected: PASS.

- [ ] **Step 5: Add `sober_header` and the banner helpers into `style.rs`**

Add private copies of `custom_banner`, `print_banner_gradient`, `lerp` to `style.rs` (as private module functions). **Do not modify `status.rs` in this task** — its `print_header` still uses its own copies, so leaving them keeps every commit compiling; Task 2 deletes the originals from `status.rs` when it removes `print_header`. Then add:

```rust
use chrono::Local;

/// Print the sober header: an optional `banner.txt` on top, then
/// `✦ Joséphine[ · suffix]` with a right-aligned clock, an optional dimmed
/// tagline, and a thin rule.
pub fn sober_header(suffix: Option<&str>, tagline: Option<&str>) {
    println!();
    if let Some(banner) = custom_banner() {
        print_banner_gradient(&banner);
        println!();
    }
    let clock = Local::now().format("%H:%M").to_string();
    let mut title = String::from("✦ Joséphine");
    if let Some(s) = suffix {
        title.push_str(&format!(" · {s}"));
    }
    if is_tty() {
        let pad = HEADER_WIDTH.saturating_sub(title.chars().count() + clock.chars().count());
        println!("{}{}{}", accent(&title), " ".repeat(pad.max(1)), clock.dimmed());
    } else {
        println!("{title}  {clock}");
    }
    if let Some(t) = tagline {
        println!("{}", if is_tty() { t.dimmed().to_string() } else { t.to_string() });
    }
    println!("{}", "─".repeat(HEADER_WIDTH).dimmed());
}
```

(`Colorize` is already imported in `style.rs`; add `use chrono::Local;` and `use colored::Color;` if missing.)

- [ ] **Step 6: Export the new primitives** in `mod.rs`

```rust
pub use style::{
    accent, check_label, confirm, format_metric_value, is_tty, primary_metric, severity_paint,
    sober_header, sparkline, status_glyph, HEADER_WIDTH,
};
```

Remove `print_banner` from the re-export only after Tasks 2–4 stop using it (see Task 4). For now keep it exported.

- [ ] **Step 7: Compile & commit**

Run: `cargo build -p josephine` → Expected: builds clean.
```bash
git add crates/josephine/src/output/style.rs crates/josephine/src/output/mod.rs
git commit -m "feat(output): shared 'Constellation sobre' primitives (glyph, accent, sober header)"
```

---

### Task 2: Rewrite `status` to "Constellation sobre"

**Files:**
- Modify: `crates/josephine/src/output/status.rs`

**Interfaces:**
- Consumes from Task 1: `status_glyph`, `severity_paint`, `sober_header`, `HEADER_WIDTH`, `is_tty`, `primary_metric`, `format_metric_value`.
- Produces: unchanged public signature `pub fn print_status_table(results: &[CheckResult])`.

**Target output** (see spec §3 `status` mockup). Rules:
- Header via `sober_header(None, Some(<tagline>))`.
- One row per check: ` {glyph}  {label:<pad>}{value}` — `label` left-padded to a fixed width (`LABEL_WIDTH = 14`), `value` painted with `severity_paint` at the row's severity. **No emoji, no right-hand tag.**
- Keep the synthesized system-load row (from the CPU check) but with **no emoji** and label from `i18n::t("Load", "Charge")`.
- Footer: a single line — count of non-`Info` checks:
  - all clear → `i18n::t("All clear.", "Tout est au vert.")`
  - otherwise → `N points à regarder → josephine doctor` / `N things to look at → josephine doctor` (singular/plural handled).

- [ ] **Step 1: Replace the header** — delete `print_header`, `header_lines`, `custom_banner`, `print_banner_gradient`, `lerp` (the last three now live in `style.rs`). In `print_status_table`, call:

```rust
pub fn print_status_table(results: &[CheckResult]) {
    super::style::sober_header(
        None,
        Some(i18n::t("Your machine, watched over.", "Votre machine, sous bonne garde.")),
    );
    for row in build_rows(results) {
        print_row(&row);
    }
    println!("{}", "─".repeat(super::style::HEADER_WIDTH).dimmed());
    print_footer_line(results);
}
```

- [ ] **Step 2: Drop the emoji & tags from rows** — replace `check_style` usage and the `Row.icon` field. New `Row`:

```rust
struct Row {
    label: String,
    value: String,
    severity: Severity,
}
```

Update `check_row` to set `label: super::style::check_label(&result.check_name).to_string()` (reuse the emoji-free labels already in `style.rs`), keep `value` logic, keep `severity`. Update `load_row` to drop `icon` and use `label: i18n::t("Load", "Charge").to_string()`.

- [ ] **Step 3: New `print_row`**

```rust
const LABEL_WIDTH: usize = 14;

fn print_row(row: &Row) {
    let glyph = super::style::status_glyph(row.severity);
    let label = pad(&row.label, LABEL_WIDTH);
    let value = super::style::severity_paint(&row.value, row.severity);
    println!(" {glyph}  {label}{value}");
}
```

Keep the existing `pad` helper. Delete `value_color`, `badge_text`, `paint`, the advice box (`print_advice`), `wrap`, and `state_badge` **only if** `doctor` no longer needs `state_badge` (it won't after Task 3 — remove it there in Task 3, so for now leave `state_badge` and delete it in Task 3). Delete `BOX_WIDTH`, `VALUE_WIDTH`, `LABEL_COLUMN`.

- [ ] **Step 4: Footer line**

```rust
fn print_footer_line(results: &[CheckResult]) {
    let n = results.iter().filter(|r| r.worst_severity() != Severity::Info).count();
    let msg = if n == 0 {
        i18n::t("All clear.", "Tout est au vert.").to_string()
    } else {
        match i18n::lang() {
            josephine_core::i18n::Lang::En => format!(
                "{n} thing{} to look at → josephine doctor", if n > 1 { "s" } else { "" }
            ),
            josephine_core::i18n::Lang::Fr => format!(
                "{n} point{} à regarder → josephine doctor", if n > 1 { "s" } else { "" }
            ),
        }
    };
    println!(" {}", if super::style::is_tty() { msg.dimmed().to_string() } else { msg });
}
```

- [ ] **Step 5: Build, run, eyeball**

Run: `cargo run -p josephine -- status`
Expected: sober `✦ Joséphine` header, coloured `●/▲/✕` rows, no emoji, no `♥`, no rounded box, one-line footer. Then `cargo run -p josephine -- status | cat` (non-TTY) → glyphs become `[ok]/[!]/[x]`, no colour.

- [ ] **Step 6: Commit**

```bash
git add crates/josephine/src/output/status.rs crates/josephine/src/output/mod.rs
git commit -m "feat(status): Constellation sobre layout — status dots, sober header, no emoji/box"
```

---

### Task 3: Rewrite `doctor` as per-check blocks

**Files:**
- Modify: `crates/josephine/src/output/doctor.rs`
- Modify: `crates/josephine/src/output/status.rs` (remove now-unused `state_badge`)
- Modify: `crates/josephine/src/output/bars.rs` (keep `bar_plain`; `severity_color` may become unused — remove if so)

**Interfaces:**
- Consumes: `status_glyph`, `severity_paint`, `sober_header`, `bar_plain`, `BAR_WIDTH`, `check_label`, `format_metric_value`, `metric_scale`, `primary_metric`, the existing `detail_lines`/`metric_label`/`fmt_threshold`/`human_interval`/`process_header` helpers (keep them).
- Produces: unchanged signature `pub fn print_doctor(results: &[CheckResult], config: &Config, verbose: bool)`.

**Target output** (see spec §3 `doctor` mockup): drop the comfy-table box entirely. For each check print a block:
```
 {glyph}  {label} · {state}{right-aligned headline value}
    {bar}  {first detail / primary line}
    {subsequent detail lines, indented 4}
```
where `state` = `i18n::t("ok"/"attention"/"critique")` coloured to severity.

- [ ] **Step 1: Replace the table body** in `print_doctor`

```rust
pub fn print_doctor(results: &[CheckResult], config: &Config, verbose: bool) {
    super::style::sober_header(
        Some(i18n::t("diagnostic", "diagnostic")),
        Some(&summary_line(results)),
    );
    for result in results {
        print_check_block(result, config, verbose);
    }
    println!();
    print_footer(footer_hint(verbose));
    println!();
}
```

- [ ] **Step 2: Add `summary_line`, `print_check_block`, `state_word`, `footer_hint`**

```rust
fn summary_line(results: &[CheckResult]) -> String {
    let total = results.len();
    let n = results.iter().filter(|r| r.worst_severity() != Severity::Info).count();
    match i18n::lang() {
        i18n::Lang::En => format!("{total} checks · {n} to look at"),
        i18n::Lang::Fr => format!("{total} contrôles · {n} à regarder"),
    }
}

fn state_word(severity: Severity) -> String {
    let w = match severity {
        Severity::Info => i18n::t("ok", "ok"),
        Severity::Attention => i18n::t("attention", "attention"),
        Severity::Critique => i18n::t("critical", "critique"),
    };
    super::style::severity_paint(w, severity)
}

fn print_check_block(result: &CheckResult, config: &Config, verbose: bool) {
    let severity = result.worst_severity();
    let glyph = super::style::status_glyph(severity);
    let label = check_label(&result.check_name);
    println!(" {glyph}  {label} · {}", state_word(severity));
    for line in detail_lines(result, config, verbose) {
        println!("    {line}");
    }
}

fn footer_hint(verbose: bool) -> &'static str {
    if verbose {
        i18n::t(
            "Condensed view: `josephine doctor` (without --verbose).",
            "Vue condensée : `josephine doctor` (sans --verbose).",
        )
    } else {
        i18n::t(
            "See everything (thresholds, processes, intervals): `josephine doctor --verbose`.",
            "Tout voir (seuils, processus, intervalles) : `josephine doctor --verbose`.",
        )
    }
}
```

Keep `detail_lines` and its helpers as-is (they already produce bar + value + details lines). Remove the comfy-table imports (`UTF8_FULL`, `Table`, `Cell`, …), `TABLE_WIDTH`, and the `state_badge` import.

- [ ] **Step 3: Remove `state_badge`** from `status.rs` (now unused) and, if `bars::severity_color` is unused after this, remove it and its `comfy_table::Color` import.

- [ ] **Step 4: Build, run, eyeball**

Run: `cargo run -p josephine -- doctor` and `cargo run -p josephine -- doctor --verbose`
Expected: per-check blocks with `●/▲/✕`, `label · state`, indented bar/detail lines, no boxed table. `| cat` → ASCII glyphs, no colour.

- [ ] **Step 5: Commit**

```bash
git add crates/josephine/src/output/doctor.rs crates/josephine/src/output/status.rs crates/josephine/src/output/bars.rs
git commit -m "feat(doctor): per-check blocks in the Constellation sobre language (drop boxed table)"
```

---

### Task 4: Restyle & detone `history`

**Files:**
- Modify: `crates/josephine/src/commands/history_cmd.rs`
- Modify: `crates/josephine/src/output/mod.rs` (drop `print_banner` export if now unused)

**Interfaces:**
- Consumes: `sober_header`, `sparkline`, `check_label`, `is_tty`.

- [ ] **Step 1: Sober header** — replace `print_banner(...)` with:

```rust
crate::output::sober_header(Some(i18n::t("24 h", "24 h")), None);
```

- [ ] **Step 2: Lighten the tables** — change both `trend` and `events_table` presets from `UTF8_FULL` to a lighter one:

```rust
use comfy_table::presets::UTF8_BORDERS_ONLY;
// ...
trend.load_preset(UTF8_BORDERS_ONLY);
// ...
events_table.load_preset(UTF8_BORDERS_ONLY);
```

- [ ] **Step 3: Detone the copy** (EN + FR), replacing the cutesy strings:

| Location | New EN | New FR |
|---|---|---|
| history disabled | `History is off. Enable it in the config and I'll keep the log.` | `Historique désactivé. Activez-le dans la configuration et je tiendrai le journal.` |
| no data yet | `No data yet. Start the daemon (\`josephine daemon start\`) and it fills in over the hours.` | `Pas encore de données. Lancez le démon (\`josephine daemon start\`) et il se remplit au fil des heures.` |
| no events (both TTY/non-TTY → single string) | `No events — a calm 24 hours.` | `Aucun événement — 24 h calmes.` |

Collapse the TTY/non-TTY branch for "no events" into one string (the `\n`-with-silence variant goes away).

- [ ] **Step 4: Build, run, eyeball**

Run: `cargo run -p josephine -- history`
Expected: sober `✦ Joséphine · 24 h` header, lighter tables, sparklines intact, detoned copy. (With no daemon data it prints the "No data yet" line — that's fine.)

- [ ] **Step 5: Commit**

```bash
git add crates/josephine/src/commands/history_cmd.rs crates/josephine/src/output/mod.rs
git commit -m "feat(history): sober header, lighter tables, detoned copy"
```

---

### Task 5: Detone desktop notifications & remaining check copy

**Files:**
- Modify: `crates/josephine-core/src/messages.rs`
- Modify: `crates/josephine-core/src/checks/kernel.rs`

**Detoning rule:** direct, calm, reassuring; short; keep the caring intent; **drop** mascot metaphors, `♥`, exclamation theatre, "between us / entre nous", "purring / ronronne", "logbook naps". Keep every string EN+FR. Keep pointing to `josephine doctor` where it already does.

**Hard test constraints (do not break):**
- `messages_never_use_alarmist_vocabulary` / `update_messages_stay_warm`: no `ERROR/FATAL/PANIC/CRASH/ÉCHEC`.
- `alerts_mention_doctor_in_both_languages`: `disk_alert` must still contain `josephine doctor`.
- `recovery_messages_are_warm`: `recovery_message("cpu", …)` must contain `"breathing"` (EN) and `"respire"` (FR) — **either keep those words, or update this test in the same commit** to the new calm wording.

**Worked examples** (apply the same spirit to every function):

| Function | Before (FR) | After (FR) |
|---|---|---|
| `kernel_alert` ok path / `kernel.rs:55-56` | `Le noyau ronronne — rien à signaler.` | `Aucun incident noyau sur la dernière heure.` (EN: `No kernel incidents in the last hour.`) |
| generic `other` alert | `Entre nous, {other} me fait un signe … Rien de grave… pour l'instant.` | `{other} sort de sa plage ({v}). Rien de grave pour l'instant — \`josephine doctor\` ?` (EN: `{other} is out of range ({v}). Nothing serious yet — \`josephine doctor\`?`) |

- [ ] **Step 1:** Rewrite each message-producing function in `messages.rs` per the rule, keeping EN+FR. Fix `kernel.rs:55-56`.

- [ ] **Step 2: Run the messages tests**

Run: `cargo test -p josephine-core messages`
Expected: PASS. If `recovery_messages_are_warm` fails because you changed the recovery wording, update that test's expected substrings in the same edit and re-run.

- [ ] **Step 3: Full core tests**

Run: `cargo test -p josephine-core`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/josephine-core/src/messages.rs crates/josephine-core/src/checks/kernel.rs
git commit -m "feat(messages): detone notifications & kernel copy to 'chaleur sobre' (EN+FR)"
```

---

### Task 6: CHANGELOG + full quality gate + manual verification

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: CHANGELOG entry** under `## [Unreleased]`

```markdown
### Changed

- **Sober CLI redesign ("Constellation sobre").** `status`, `doctor` and
  `history` drop the ASCII-angel banner, per-check emoji, hearts and the rounded
  advice box for a cleaner layout: a discreet `✦` header, status carried by
  coloured shape-glyphs (`●` ok · `▲` attention · `✕` critical, degrading to
  `[ok]/[!]/[x]` off a terminal), aligned columns and a one-line footer. The
  guardian-angel voice stays warm but becomes direct and concise, in both
  English and French — including the desktop notifications. A custom
  `banner.txt` is still honoured.
```

- [ ] **Step 2: Format & lint**

Run: `cargo fmt --all` then `cargo fmt --check`
Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean (fix any unused-import/dead-code warnings from removed helpers).

- [ ] **Step 3: Full test suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 4: Manual verification (TTY + non-TTY)**

Run each and confirm against the spec mockups:
`cargo run -p josephine -- status` · `... doctor` · `... doctor --verbose` · `... history`
Then pipe one through `| cat` and confirm `[ok]/[!]/[x]` + no colour + no stray ANSI.

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): note the Constellation sobre CLI redesign"
```

---

## Self-Review

**Spec coverage:**
- Tone "chaleur sobre" → Tasks 2–5 (copy) ✅
- Visual language (glyphs, header, palette, non-TTY fallback) → Task 1 + applied in 2–4 ✅
- `status` / `doctor` / `history` re-layout → Tasks 2 / 3 / 4 ✅
- `messages.rs` in scope → Task 5 ✅
- `banner.txt` still honoured → Task 1 `sober_header` ✅
- Non-TTY plain fallback → Task 1 glyph + eyeball steps ✅
- i18n EN+FR everywhere → every copy step is a pair ✅
- Tests updated → Task 1 unit test, Task 5 test constraints, Task 6 full gate ✅
- Acceptance criteria 1–5 → covered by Tasks 2–4 (1), 2–5 (2), 1 (3), 1 (4), 6 (5) ✅

**Placeholder scan:** No TBD/TODO; copy is given verbatim or via an explicit rule + worked examples for the bulk `messages.rs` rewrite (the strings are the deliverable and live in-file).

**Type consistency:** `status_glyph`/`severity_paint`/`accent`/`sober_header`/`HEADER_WIDTH` are defined in Task 1 and consumed with those exact names/signatures in Tasks 2–4. `Severity` variants (`Info`/`Attention`/`Critique`) match `check.rs`.

## Notes / out of scope
- The inode false-positive fix lives on `fix/inode-squashfs-false-positive`; independent, merges cleanly.
- `fix_cmd.rs` also has one cutesy line (`… file un parfait bonheur ✨`); it's not one of the three surfaces, so it's left for increment C's copy sweep unless trivially included.
- `report` output is unchanged (increment A is `status`/`doctor`/`history` + notifications only).
