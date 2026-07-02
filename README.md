<p align="center">
  <img src="resources/social-preview-en.png" alt="Joséphine — your computer's quiet guardian angel" width="720">
</p>

<h1 align="center">Joséphine</h1>

<p align="center"><em>Your computer's quiet guardian angel.</em></p>

<p align="center">
  <a href="https://github.com/systm-d/josephine/actions/workflows/ci.yml"><img src="https://github.com/systm-d/josephine/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/systm-d/josephine/releases/latest"><img src="https://img.shields.io/github/v/release/systm-d/josephine?color=e0a458&label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/platform-Linux-333" alt="Linux only">
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0"></a>
</p>

Joséphine watches your machine silently and only speaks up when it actually
helps. She keeps an eye on CPU, memory, disk, temperature, systemd services and
pending updates, detects trouble early, and sends warm, plain-language desktop
notifications — never intrusive, always local. **No data ever leaves your
computer.**

> Joséphine speaks **English by default**; set `language: fr` in
> `~/.config/josephine/config.yaml` for her French voice. The warm
> guardian-angel tone is preserved in both.

## Features

- **Eleven built-in checks** — CPU, memory, disk, temperature, systemd services,
  package updates (apt / dnf / pacman), local network (gateway latency), battery,
  inode usage, SMART disk health (opt-in) and kernel incidents (OOM / oops).
- **Warm notifications** — plain-language desktop alerts that escalate only when
  it helps; never `ERROR` / `FATAL` / `PANIC`.
- **Background daemon** — a lightweight systemd *user* service that watches
  continuously and records a rolling **24-hour history** (local SQLite).
- **At-a-glance `status`** — colour-coded summary with a **customizable banner**.
- **Detailed `doctor`** — check-by-check diagnostics; `--verbose` adds thresholds,
  the top 10 processes and each check's collection interval.
- **Self-update** — `josephine update` checks GitHub Releases and installs the
  package matching your install (`.deb` / `.rpm`); reaches the network only when
  you ask.
- **Bilingual** — English by default, French with `language: fr` in the config;
  warm, never alarmist, in both.
- **100% local** — no cloud, no telemetry, Linux-native (systemd, `/sys`,
  libnotify).

## Installation

Grab a package from the [latest release](https://github.com/systm-d/josephine/releases/latest):

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Or build from source (requires Rust 1.85+):

```sh
cargo install --git https://github.com/systm-d/josephine josephine
```

## Usage

```sh
josephine               # quick status summary (default)
josephine status        # CPU, memory, disk, temperature, systemd, updates at a glance
josephine doctor        # detailed diagnostics, check by check
josephine doctor -v     # verbose: thresholds, top 10 processes, intervals
josephine history       # last 24 h: min/avg/max + sparkline trends, and events
josephine daemon start  # run the background watcher
josephine daemon status # daemon state (PID, uptime)
josephine config show   # print the current configuration
josephine config edit   # edit the config in $EDITOR, then re-validate
josephine report        # dated plain-text health report (-o writes to a file)
josephine clean         # preview reclaimable disk space (--apply clears caches)
josephine fix           # guided remediation for failed services / low disk
josephine notify test   # send a test desktop notification
josephine update        # check GitHub for a newer version and install it
josephine --version
```

`josephine update` reaches the network only when you run it — never in the
background. It detects how Joséphine was installed (`.deb`/`.rpm`/…), downloads
the matching package, verifies its checksum, and hands the privileged install
step (`sudo`) to you.

To keep Joséphine watching across reboots, enable the bundled systemd **user**
unit ([`packaging/systemd/josephine.service`](packaging/systemd/josephine.service)):

```sh
systemctl --user enable --now josephine
```

Configuration lives at `~/.config/josephine/config.yaml` (created on first run).
History and the daemon's state live under `~/.local/share/josephine/`.

The `status` header is deliberately sober. Want a flourish? Drop any ASCII/Braille
art in `~/.config/josephine/banner.txt` and it appears above the title, tinted
with a gradient. A ready-to-use example lives at [`resources/banner.txt`](resources/banner.txt).

## Documentation

- [Architecture](docs/ARCHITECTURE.md) · [Current state](docs/CURRENT_STATE.md) · [Roadmap](docs/ROADMAP.md)
- [Contributing](CONTRIBUTING.md) · [Conventions](CONVENTIONS.md) · [Code of Conduct](CODE_OF_CONDUCT.md) · [Security](SECURITY.md)
- Website: <https://systm-d.github.io/josephine/>

## Development

```sh
cargo build
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and quality gate.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
