# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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

## [0.1.0] - 2026-06-30

### Added

- Initial release: `status`, `doctor`, `history`, `daemon`, `config` commands.
- Five checks: cpu, memory, disk, temperature, systemd.
- Background daemon with desktop notifications and a 90-day SQLite history.
