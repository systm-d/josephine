# Joséphine — project guide for AI agents & contributors

Local Linux system guardian. Rust workspace: `josephine-core` (pure logic) +
`josephine` (binary, thin CLI).

## Read first

1. [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md) — what exists
2. [docs/ROADMAP.md](docs/ROADMAP.md) — priorities
3. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — how the code is organized
4. [CONVENTIONS.md](CONVENTIONS.md) — shared standards (edition, fmt, lints, commits)
5. [CONTRIBUTING.md](CONTRIBUTING.md) — workflow & quality gate

## Product rules

- French, warm tone (*Joséphine, guardian angel*); never `ERROR`/`FATAL`/`PANIC`
  in user-facing text.
- 100% local, no cloud.
- Linux-only (systemd, `/sys/class/thermal`, libnotify).

## Where to change what

| Need | File |
|------|------|
| New check | `crates/josephine-core/src/checks/` + `config.rs` + `messages.rs` |
| Notification text | `crates/josephine-core/src/messages.rs` |
| CLI output | `crates/josephine/src/output/` |
| CLI command | `crates/josephine/src/commands/` |
| DB schema | `crates/josephine-core/migrations/` (versioned, `schema_version`) |

## Quality gate

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p josephine -- status
```
