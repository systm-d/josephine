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
