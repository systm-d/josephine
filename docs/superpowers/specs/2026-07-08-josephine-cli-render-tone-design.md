# Joséphine — Increment A: CLI render & tone ("Constellation sobre")

**Date:** 2026-07-08
**Status:** Design approved, spec under review
**Scope:** Increment A of the CLI revamp program (A → C → B → D)
**Branch:** `feat/cli-render-tone`

---

## Context

Joséphine's CLI output reads as *"nœud-nœud"* — cutesy/twee. The big Braille
angel banner, per-check emoji, `♥` hearts and mascot-y copy ("Le noyau
ronronne…") undercut the tool's credibility. The user wants a more direct,
grown-up look inspired by their own app [terminus-32](https://exec-d.github.io/terminus-32/)
— which is minimal, honest and *"sans langue de bois"* (plain-spoken), **not**
neon/retro-terminal. Joséphine's stellar/guardian identity stays; the sugar goes.

This increment is the foundation the rest of the program inherits: later work
(`--json` in increment B) depends on the output structure defined here.

### Decisions locked during brainstorming

- **Tone:** *"chaleur sobre"* — keep the caring guardian intent, make it direct
  and concise, drop the ornaments. Stays within the `CLAUDE.md` product rule
  (warm tone, never `ERROR`/`FATAL`/`PANIC`).
- **Visual ambition:** *re-layout sobre* — rethink the layout, not a full
  reinvention.
- **Direction:** **A — "Constellation sobre"** (colored status dots, no emoji,
  aligned columns, a discreet `✦` as the only stellar wink).
- **Notifications** (`messages.rs`, desktop): **in scope** — detone for
  consistency.
- **Palette:** soft indigo/violet accent (stellar), semantic green/amber/red.

---

## Goals

1. Replace the cutesy visual language of `status`, `doctor`, `history` with the
   "Constellation sobre" language.
2. Detone all user-facing copy on those surfaces **and** the desktop
   notification messages, in both EN and FR.
3. Keep every string bilingual (`i18n::t(en, fr)` / `match i18n::lang()`), keep
   the non-TTY plain fallback, keep custom `banner.txt` support.

## Non-goals (belong to later increments)

- `--json` / machine-readable output (B).
- Shell completions, localized `--help`, terminal-channel notifications (B).
- New checks or commands (D).
- The inode false-positive fix (already done on `fix/inode-squashfs-false-positive`).
- The website redesign (separate track; keeps its "stellaire" look).

---

## Design

### 1. Tone — "chaleur sobre"

Principle: **direct, honest, reassuring — without mush.** Short sentences.
No `♥`, no mascot metaphors, no exclamation theatre. Still human, never a raw
`ERROR`. Every changed string stays EN+FR.

Representative before → after:

| Surface | Before | After (FR) | After (EN) |
|---|---|---|---|
| `status` tagline | `Je veille sur votre machine… ♥` | `Votre machine, sous bonne garde.` | `Your machine, watched over.` |
| `status` footer | boxed `╭─╮ 💬 Un point mérite votre attention…` | `3 points à regarder → josephine doctor` | `3 things to look at → josephine doctor` |
| kernel ok detail | `Le noyau ronronne — rien à signaler.` | `Aucun incident noyau sur la dernière heure.` | `No kernel incidents in the last hour.` |
| error prefix | `✨ Joséphine a rencontré un souci : …` | *unchanged* (already within tone) | *unchanged* |

The full copy inventory (every touched string) is produced during
implementation from a grep of literals in the affected modules; each gets an
EN+FR rewrite reviewed against this principle.

### 2. Visual language ("Constellation sobre")

- **Status glyphs** carry severity by **shape *and* colour** (readable without
  colour, colour-blind-safe):
  - `●` green — ok
  - `▲` amber — attention (warning)
  - `✕` red — critique (critical)
  - `·` dim — unavailable / informational
  They replace **both** the per-check emoji **and** the `[OK]/[!]/[✗]` text tags.
- **Header:** `✦ Joséphine` + right-aligned time, a one-line tagline, a thin
  `───` rule. `✦` is the only stellar ornament. A custom
  `~/.config/josephine/banner.txt` is **still honoured** (printed above the
  header) for users who want their own art; the default is the sober header.
- **Layout:** left-aligned label column, values aligned at a fixed column. No
  heavy box-drawing; a thin rule separates header/body/footer.
- **Footer:** a single action line (e.g. `N points à regarder → josephine
  doctor`), no rounded box. When all is well: `Tout est au vert.` /
  `All clear.`
- **Non-TTY / plain mode:** `● ▲ ✕` → `[ok] [!] [x]`, no colour, no rule glyphs
  beyond ASCII.
- **Palette (confirmed):** accent = soft indigo/violet for
  `✦` + titles; dim grey for secondary text; green/amber/red semantic for
  status. Defined centrally in `output/style.rs`.

### 3. Surfaces

`status` (flagship):

```
✦ Joséphine                                      14:40
Votre machine, sous bonne garde.
──────────────────────────────────────────────────────
 ●  CPU            5 %
 ●  Charge         0.47 · 0.76 · 1.27
 ●  Mémoire        27 %   8.3G / 31G
 ▲  Disque         85 %   « / » 403G / 475G
 ●  Température     60 °C
 ▲  Services       1 en échec
 ●  Mises à jour   à jour
 ▲  Réseau         passerelle injoignable
 ●  Batterie       100 %   (branchée)
 ●  Inodes         1 %
 ●  Noyau          0 incident (1 h)
──────────────────────────────────────────────────────
 3 points à regarder → josephine doctor
```

The right-hand `ok/!` tag column is dropped — the leading dot already carries
state.

`doctor` — same codes, one block per check, no boxed table:

```
✦ Joséphine · diagnostic                         14:40
11 contrôles · 3 à regarder
──────────────────────────────────────────────────────
 ▲  Disque · attention                          84.9 %
    ▓▓▓▓▓▓▓▓▓▓▓▓▓░░  « / » (btrfs) 403.4 / 475.4 G · SSD
    /boot 56.5 %  ·  /boot/efi 3.2 %
 ▲  Services · attention                     1 en échec
    clamd@scan.service
    systemd-journald.service · 1 redémarrage
 ●  Noyau · ok                                0 incident
    Aucun incident noyau sur la dernière heure.
```

`--verbose` keeps its extra data (numeric thresholds, top-10 processes,
intervals) in the same block style.

`history` — sparklines stay (data, not sugar):

```
✦ Joséphine · 24 h                               14:40
──────────────────────────────────────────────────────
             min    moy    max   tendance
CPU          2 %    9 %   61 %   ▁▂▃▂▅▇▃▂
Mémoire     24 %   27 %   38 %   ▃▃▄▃▅▄▃▃
Température  41°C   52°C   68°C   ▂▃▅▇▅▃▂▁

Événements
 ▲ 03:12  Réseau — passerelle injoignable (rétabli 03:18)
```

---

## Architecture — where it changes

| File | Change |
|---|---|
| `crates/josephine/src/output/style.rs` | Status glyphs, severity→glyph/colour map, indigo/violet palette, non-TTY fallback, `✦` accent. Single source of truth for the visual language. |
| `crates/josephine/src/output/status.rs` | New `status` layout. |
| `crates/josephine/src/output/doctor.rs` | New per-check block layout (drop comfy-table box). |
| `crates/josephine/src/output/bars.rs` | Slim bar restyle; removed from `status` (kept in `doctor`). |
| `crates/josephine/src/commands/history_cmd.rs` | New `history` layout (keep sparklines). |
| `crates/josephine/src/commands/status_cmd.rs`, `doctor_cmd.rs` | Header/footer/tagline copy. |
| `crates/josephine-core/src/messages.rs` | Detone desktop notification copy (EN+FR). |
| `crates/josephine-core/src/checks/*.rs` | Detone the few cutesy `status_value`/detail strings (e.g. `kernel.rs`). Keep `i18n::t`. |

Notes:
- comfy-table may still be used where a real table helps (`history`), or dropped
  entirely — decided during implementation; the box-drawing *look* goes either
  way.
- Colour is applied via the table/style API, never raw ANSI in cell text
  (existing convention).

## i18n

Every touched or new string ships EN + FR via `i18n::t(en, fr)` or
`match i18n::lang()`. No English-only literals. The glyphs and layout are
language-independent; only the words change per language.

## Testing

- Update `assert_cmd` integration tests that assert on output substrings (e.g.
  the old tagline / footer / `[OK]` tags) to the new strings.
- Add assertions that the non-TTY plain output uses `[ok] [!] [x]` and no ANSI.
- Add a small unit test for the severity→glyph mapping in `style.rs`.
- Quality gate unchanged: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, plus a manual `cargo run -p josephine -- status`.

## Backward compatibility

- `banner.txt` still honoured — no breaking change for users with custom art.
- Config schema untouched.
- Independent of the inode-fix branch (different files); the two merge cleanly.
  Only possible overlap: detoned `status_value` strings in a check the inode fix
  also touches — trivial to reconcile.

## Acceptance criteria

1. `status`, `doctor`, `history` render in the "Constellation sobre" language:
   status dots (shape+colour), sober `✦` header, aligned columns, thin rules,
   one-line footer, no emoji, no `♥`, no rounded box.
2. All user-facing copy on these surfaces and in `messages.rs` is detoned, EN+FR.
3. Non-TTY output degrades to `[ok]/[!]/[x]`, no colour.
4. Custom `banner.txt` still works.
5. Full quality gate green.

## Follow-ups (not this increment)

- Increment C: ship the inode fix; refresh stale `CURRENT_STATE`/`ROADMAP`.
- Increment B: `--json` builds on the output structure defined here.
- Consider updating the website's example output screenshots once this lands.
