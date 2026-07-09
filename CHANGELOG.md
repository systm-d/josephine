# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- The whole CLI now reads as one system: the remaining commands (`report`,
  `clean`, `fix`, `update`, `daemon`, `config`, `notify`) drop the old
  `✨ Joséphine` banners and mascot copy for the sober `✦` header and the
  direct "chaleur sobre" voice (English + French); `status` and `doctor` share
  one footer; `history` events show human check labels. The `CURRENT_STATE`
  and `ROADMAP` docs are refreshed to the 0.7.0 baseline.

## [0.7.0] - 2026-07-09

### Changed

- **Sober CLI redesign ("Constellation sobre").** `status`, `doctor` and
  `history` drop the ASCII-angel banner, per-check emoji, hearts and the rounded
  advice box for a cleaner layout: a discreet `✦` header, status carried by
  coloured shape-glyphs (`●` ok · `▲` attention · `✕` critical, degrading to
  `[ok]/[!]/[x]` off a terminal), aligned columns and a one-line footer. The
  guardian-angel voice stays warm but becomes direct and concise, in both
  English and French — including the desktop notifications. A custom
  `banner.txt` is still honoured.

### Fixed

- The **inode** check no longer raises a false `critical` from snap mounts. Snaps
  are read-only `squashfs` images, always packed to 100 % inode usage by design;
  on most distros snapd mounts them under `/var/lib/snapd/snap/…`, which the old
  path-based filter (matching only `/snap`) missed, so a machine with many snaps
  would drown the real writable filesystems under dozens of 100 % lines. The
  check now reads `df -iPT` and skips read-only image filesystems by *type*
  (`squashfs` / `iso9660` / `erofs`), wherever they mount.

## [0.6.0] - 2026-07-03

### Added

- Each release now ships ready-to-use packaging recipes as assets: a Homebrew
  formula (`josephine.rb`) and an AUR `PKGBUILD`. The release workflow fills in
  the tag's version and the source tarball's real `sha256` in both, so on Linux
  you can
  `brew install https://github.com/systm-d/josephine/releases/latest/download/josephine.rb`
  (or `makepkg` from the attached `PKGBUILD`).

## [0.5.0] - 2026-07-02

### Added

- **Internationalisation.** Every user-facing string now ships in **English and
  French**: notifications, `status` / `doctor` / `history` output, all commands,
  check values, and config-validation errors. Set `language: fr` in the config
  for the French voice. The warm guardian-angel tone is preserved in both.

### Changed

- **English is now the default language** (Joséphine was French-only). French is
  one `language: fr` config line away. `CLAUDE.md`, README and docs updated.
- Landing site redesigned — warmer, livelier (halo hero, gradient feature cards,
  a commands reference, terminal chrome) and bilingual (EN default + FR), with
  real, anonymised example output.

## [0.4.1] - 2026-07-02

### Fixed

- `josephine daemon start` no longer refuses to start when the recorded PID has
  been recycled by an unrelated process. Daemon liveness now verifies the
  process is actually Joséphine (via `/proc/<pid>/cmdline`), so a stale pid file
  after a crash or logout no longer blocks a restart — which had silently
  starved `josephine history` of data. (#16)

## [0.4.0] - 2026-07-01

### Added

- Three new "guardian" checks, bringing the total to eleven:
  - **inode** — flags filesystems low on inodes (a disk can be "full" on inodes
    while still showing free space). Reads `df -iP`; runs as a normal user.
  - **smart** — per-disk SMART self-assessment via `smartctl`, an early warning
    of drive failure. Off by default (needs root); degrades to an informational
    "unavailable" rather than a false alarm.
  - **kernel** — counts recent kernel incidents (OOM kills, oops, BUG, panic)
    from `journalctl -k`, degrading gracefully when the journal isn't readable.
- Richer `josephine history`: per-metric **min / avg / max** and a 24-hour
  **sparkline** trend (`▁▂▃▅▇`) for CPU, memory, disk, temperature, network and
  battery — instead of only the daily maximum.

## [0.3.2] - 2026-07-01

### Changed

- `josephine update` stages the downloaded package under `/var/tmp` (in an
  owner-only-writable, ownership-checked subdirectory) instead of `$HOME`, so
  apt's sandboxed `_apt` user can read it — no more "Download is performed
  unsandboxed" warning during install. The package stays unreadable-for-write to
  other users, keeping the verify-then-install path safe on shared machines.

## [0.3.1] - 2026-07-01

### Changed

- `status` disk line is easier to read: `20% de « / » (192G / 937G)` — the
  percentage comes first (like the other checks) and the mount point is quoted
  so it can't be mistaken for a separator.

## [0.3.0] - 2026-07-01

### Added

- Two new checks, bringing the total to eight:
  - **network** — default-gateway reachability and latency. Stays strictly local
    (pings the LAN gateway, reads `/proc/net/route` and `/etc/resolv.conf`); no
    external host is ever contacted.
  - **battery** — charge level, charging state and health from
    `/sys/class/power_supply`; only warns while discharging and low, and reports
    "no battery" gracefully on desktops.
- `josephine report` — a dated, plain-text health snapshot, printed or written to
  a file with `--output`.
- `josephine clean` — previews reclaimable space (user cache, thumbnails, `/tmp`,
  systemd journals) by default; `--apply` clears the always-safe thumbnail cache
  and shows the exact commands for the privileged reclaims.
- `josephine fix` — guided remediation: surfaces failed services and disk
  pressure with the precise command to fix each. Advisory only; nothing
  privileged runs on its own.
- `josephine notify test` — sends a test desktop notification to verify libnotify.
- `josephine config edit` — opens the config in `$EDITOR`, then re-validates it.

### Changed

- Replaced the `clean`/`fix`/`report` and `config edit` stubs with working
  commands.
- Roadmap and current-state docs refreshed to the 0.3.0 baseline; dropped an
  obsolete cargo-deny advisory exception that no longer matched any crate.

## [0.2.2] - 2026-07-01

### Added

- `josephine update`: checks GitHub Releases and, on request, downloads and
  installs the package matching your install channel (`.deb`/`.rpm`), leaving the
  privileged step (`sudo`) to you. The network is touched only on this explicit
  command — never in the background. `--check` reports without installing;
  `--yes` skips the confirmation prompt. Release artifacts now ship a `.sha256`
  per package so the download can be integrity-checked.

## [0.2.1] - 2026-07-01

### Changed

- `status` header is now a sober title block; removed the built-in ASCII-art
  angel avatar. A custom `<config>/banner.txt` is still honoured when present.

## [0.2.0] - 2026-07-01

### Added

- New `updates` check: counts available package updates via apt / dnf / pacman.
- Redesigned `status` screen: angel banner, per-check emoji icons, a system load
  line (from `/proc/loadavg`), colour-coded values and a rounded advice box.
- Customizable banner: `status` uses `<config>/banner.txt` (any ASCII/Braille
  art, tinted with a gradient) when present, else the built-in angel.
- `josephine doctor --verbose`: adds numeric thresholds, the top 10 processes
  (CPU & memory) and each check's collection interval.
- Project landing site (Zola) deployed to GitHub Pages.
- `josephine daemon run` foreground subcommand for systemd supervision.
- Packaging: systemd user unit, deb/rpm metadata, AUR PKGBUILD, Homebrew (Linux)
  formula, and a tag-driven release workflow (GitHub Releases + crates.io).
- Committed `Cargo.lock` for reproducible builds.
- Continuous integration: lint, multi-distro test matrix (Ubuntu 22.04/24.04,
  Fedora 40/41), coverage (informational), supply-chain security checks, and a
  criterion benchmark.

### Changed

- Migrated the workspace to Rust edition 2024 (MSRV 1.85).
- Renamed the binary crate `josephine-cli` to `josephine`.
- Dual-licensed under MIT OR Apache-2.0.
- Adopted the shared `rust-cli-template` conventions (toolchain, rustfmt, lints,
  release profile, integration tests).
- Reworked the CLI copy (stubs, `config`, daemon lifecycle, status/doctor/history
  footers) for a warmer, wittier guardian-angel voice.
- `josephine doctor` now renders a single unified table (one row per check)
  instead of one box per check.

### Fixed

- `josephine config edit` now prints a friendly "coming soon" message and exits 0,
  instead of surfacing as an error (exit 1) like the other stub commands.

## [0.1.0] - 2026-06-30

### Added

- Initial release: `status`, `doctor`, `history`, `daemon`, `config` commands.
- Five checks: cpu, memory, disk, temperature, systemd.
- Background daemon with desktop notifications and a 90-day SQLite history.
