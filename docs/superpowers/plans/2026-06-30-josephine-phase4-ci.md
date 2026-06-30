# Joséphine — Phase 4: CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a GitHub Actions CI pipeline (lint, multi-distro test matrix, coverage, supply-chain security, bench-smoke) plus the criterion benchmark and tarpaulin config it needs, so every push/PR to `main` is verified.

**Architecture:** One workflow `.github/workflows/ci.yml` with five independent jobs. Tests run on a Linux-only matrix (Ubuntu 22.04/24.04 native + Fedora 40/41 in containers, with Rust installed via rustup so the edition-2024/MSRV-1.85 toolchain is used rather than the distro's older packaged Rust). Coverage is **informational** (tarpaulin + Codecov, with IO-bound modules excluded and an 80% target documented) so CI stays green while the test suite is still light. A small criterion bench in `josephine-core` is compile-smoke-tested to keep it from rotting.

**Tech Stack:** GitHub Actions, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `taiki-e/install-action`, `cargo-tarpaulin`, `cargo-audit`, `cargo-deny`, `criterion`, Codecov.

## Global Constraints

- **Linux-only** matrix: Ubuntu 22.04, Ubuntu 24.04, Fedora 40, Fedora 41. No macOS/Windows.
- Toolchain in CI must honor edition 2024 / MSRV 1.85 → install Rust via `rustup` (NOT distro packages, which are older than 1.85 on Fedora 40/41).
- Lint job runs `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Security job runs `cargo audit` and `cargo deny check` (uses the `deny.toml` from Phase 3).
- Coverage is **not a hard gate** in this phase: `tarpaulin.toml` excludes IO modules and documents the 80% target; Codecov upload uses `fail_ci_if_error: false`. Do not add `--fail-under` to the CI invocation.
- Triggers: `push` and `pull_request` to `main`.
- No Rust source (`.rs`) logic changes; only add a bench file + manifest entries.
- This is mostly GitHub-side config: it is fully validated only by pushing. Locally, validate YAML well-formedness and (if `actionlint` is available) lint the workflow; ensure `cargo bench --no-run` compiles.
- Branch: `chore/template-alignment-p3-6` (continues after Phase 3).

---

### Task 1: criterion benchmark in josephine-core

**Files:**
- Modify: `Cargo.toml` (workspace `[workspace.dependencies]`: add `criterion`)
- Modify: `crates/josephine-core/Cargo.toml` (`[dev-dependencies]` + `[[bench]]`)
- Create: `crates/josephine-core/benches/rules.rs`

**Interfaces:**
- Consumes: `josephine_core::Config` (public; has `Default` and a `validate()` returning a `Result`).
- Produces: a `cargo bench`-runnable benchmark named `rules`.

- [ ] **Step 1: Add criterion to workspace dependencies in root `Cargo.toml`**

In `[workspace.dependencies]` add:

```toml
criterion = "0.5"
```

- [ ] **Step 2: Wire it into `crates/josephine-core/Cargo.toml`**

Append (the crate already has a `[lints]` section; add these above or below it):

```toml
[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "rules"
harness = false
```

- [ ] **Step 3: Create the benchmark `crates/josephine-core/benches/rules.rs`**

This benches pure, IO-free logic (default config construction + validation). Verify the public API first (`Config::default()` and `config.validate()` exist — they are exercised by the `default_config_is_valid` unit test); if `validate` takes/returns something slightly different, adjust this call minimally.

```rust
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use josephine_core::Config;

fn config_default_validate(c: &mut Criterion) {
    c.bench_function("config_default_validate", |b| {
        b.iter(|| {
            let cfg = Config::default();
            black_box(cfg.validate()).ok();
        });
    });
}

criterion_group!(benches, config_default_validate);
criterion_main!(benches);
```

- [ ] **Step 4: Verify it compiles and runs as a smoke test**

Run:
```bash
cargo bench --no-run
cargo bench -p josephine-core --bench rules -- --warm-up-time 1 --measurement-time 1 2>/dev/null | tail -5 || true
```
Expected: `cargo bench --no-run` compiles cleanly. Then confirm the normal gate:
```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```
Expected: PASS (criterion is dev-only, so clippy `--all-targets` now also lints the bench — keep it warning-free).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/josephine-core/Cargo.toml crates/josephine-core/benches/rules.rs
git commit -m "test: add criterion benchmark for josephine-core (bench-smoke target)"
```

---

### Task 2: tarpaulin coverage config

**Files:**
- Create: `tarpaulin.toml`

**Interfaces:**
- Produces: coverage config consumed by `cargo tarpaulin` in CI.

- [ ] **Step 1: Create `tarpaulin.toml`**

Exclude the IO-bound modules that cannot be unit-tested without real hardware, so the reported coverage reflects the testable pure-logic core. The 80% target is documented but NOT enforced yet (no `fail_under`) — ratchet it up as the suite grows.

```toml
# cargo-tarpaulin coverage config.
# Target: 80% on the testable pure-logic core (documented goal, not yet enforced —
# enforce via `fail_under` once the IO modules gain tests; see ROADMAP).
[report]
out = ["Xml", "Stdout"]

[default]
workspace = true
# IO-bound modules excluded: they read real hardware / spawn the daemon / draw
# terminal UI and cannot be unit-tested deterministically in CI.
exclude-files = [
    "crates/josephine-core/src/checks/*",
    "crates/josephine-core/src/daemon.rs",
    "crates/josephine-core/src/scheduler.rs",
    "crates/josephine-core/src/notify.rs",
    "crates/josephine/src/output/*",
    "crates/josephine/src/main.rs",
    "crates/josephine/src/cli.rs",
    "crates/josephine/tests/*",
    "crates/josephine-core/benches/*",
]
```

- [ ] **Step 2: (Optional local) sanity-check if tarpaulin is installed**

Run:
```bash
if command -v cargo-tarpaulin >/dev/null 2>&1; then cargo tarpaulin --print-summary || true; else echo "tarpaulin not installed; CI will run it"; fi
```
Expected: prints a coverage summary, or the "not installed" note. No gate here.

- [ ] **Step 3: Commit**

```bash
git add tarpaulin.toml
git commit -m "ci: add cargo-tarpaulin coverage config (IO modules excluded)"
```

---

### Task 3: CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `deny.toml` (Phase 3), `tarpaulin.toml` (Task 2), the `rules` bench (Task 1).

- [ ] **Step 1: Create `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always

jobs:
  lint:
    name: lint (fmt + clippy)
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: test (${{ matrix.name }})
    runs-on: ${{ matrix.os }}
    container: ${{ matrix.container }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - { name: ubuntu-22.04, os: ubuntu-22.04, container: "" }
          - { name: ubuntu-24.04, os: ubuntu-24.04, container: "" }
          - { name: fedora-40, os: ubuntu-24.04, container: "fedora:40" }
          - { name: fedora-41, os: ubuntu-24.04, container: "fedora:41" }
    steps:
      - name: Install container prerequisites (Fedora)
        if: matrix.container != ''
        run: dnf -y install gcc git curl
      - uses: actions/checkout@v4
      - name: Install Rust via rustup (Fedora container)
        if: matrix.container != ''
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
          echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
      - name: Install Rust (Ubuntu runners)
        if: matrix.container == ''
        uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        if: matrix.container == ''
      - run: cargo test --workspace

  coverage:
    name: coverage (informational)
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-tarpaulin
      - run: cargo tarpaulin --workspace --out Xml
      - uses: codecov/codecov-action@v4
        with:
          token: ${{ secrets.CODECOV_TOKEN }}
          fail_ci_if_error: false

  security:
    name: security (audit + deny)
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-audit,cargo-deny
      - run: cargo audit
      - run: cargo deny check

  bench-smoke:
    name: bench-smoke
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo bench --no-run
```

- [ ] **Step 2: Validate the workflow YAML locally**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('yaml ok')"
command -v actionlint >/dev/null 2>&1 && actionlint .github/workflows/ci.yml || echo "actionlint not installed; CI structure will be validated by GitHub on push"
```
Expected: `yaml ok`; actionlint clean if available.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions pipeline (lint, test matrix, coverage, security, bench-smoke)"
```

---

### Task 4: README badge + CHANGELOG note

**Files:**
- Modify: `README.md` (the CI badge added in Phase 1 already points at `ci.yml` — confirm it resolves now)
- Modify: `CHANGELOG.md` (`[Unreleased]` → add CI entry)

- [ ] **Step 1: Confirm the README CI badge target matches the workflow filename**

The Phase 1 README badge references `.github/workflows/ci.yml`. Verify it matches the file just created (it does — `ci.yml`). No edit needed unless the path differs. Run:
```bash
grep -n "workflows/ci.yml" README.md
```
Expected: the badge line is present and references `ci.yml`.

- [ ] **Step 2: Add a CHANGELOG entry under `[Unreleased]` → `### Added`**

Add this bullet under the existing `[Unreleased]` section (create an `### Added` subsection if not present):

```markdown
- Continuous integration: lint, multi-distro test matrix (Ubuntu 22.04/24.04,
  Fedora 40/41), coverage (informational), supply-chain security checks, and a
  criterion benchmark.
```

- [ ] **Step 3: Commit**

```bash
git add README.md CHANGELOG.md
git commit -m "docs: note CI in CHANGELOG"
```

---

## Done criteria for Phase 4

- `.github/workflows/ci.yml` present with five jobs; YAML valid.
- `tarpaulin.toml` present with IO-module exclusions; coverage informational (no hard gate).
- `crates/josephine-core/benches/rules.rs` present; `cargo bench --no-run` compiles.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all pass locally.
- CHANGELOG `[Unreleased]` notes CI.

## Notes / GitHub-side verification (after push)

- The CI run itself is only validated on GitHub. Likely first-push iterations: the Fedora-container Rust install, or `cargo audit` surfacing a transitive advisory (a real finding to address, not a CI-config bug).
- Codecov needs a `CODECOV_TOKEN` repo secret for non-tokenless upload; `fail_ci_if_error: false` keeps the job green meanwhile.
- Enforce a coverage floor (`fail_under` in `tarpaulin.toml`) only after the IO modules gain tests (the deferred test-hardening item).
