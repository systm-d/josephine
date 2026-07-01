# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
