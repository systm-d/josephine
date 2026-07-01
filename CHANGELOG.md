# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
