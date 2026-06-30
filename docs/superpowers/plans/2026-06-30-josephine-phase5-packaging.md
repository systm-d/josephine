# Joséphine — Phase 5: Packaging & Release (Linux-pragmatic) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Joséphine installable and releasable on Linux — a committed lockfile, a clean public foreground daemon entry for systemd, a systemd **user** unit, deb/rpm Cargo metadata, AUR + Homebrew formulas, and a tag-driven `release.yml` that builds artifacts and publishes to GitHub Releases + crates.io.

**Architecture:** Linux-only packaging (no Windows/macOS). A small public `josephine daemon run` subcommand exposes the foreground watcher (today only reachable via the hidden `--__daemon__` flag) so the systemd unit calls a stable command. `Cargo.lock` is committed so `--locked` builds are reproducible across packaging. deb/rpm are driven by `[package.metadata.deb]` / `[package.metadata.generate-rpm]` on the binary crate; AUR/Homebrew build from the release source tarball. `release.yml` runs on `v*.*.*` tags.

**Tech Stack:** clap, `cargo-deb`, `cargo-generate-rpm`, AUR PKGBUILD, Homebrew (Linux), GitHub Actions, `softprops/action-gh-release`, crates.io.

## Global Constraints

- **Linux-only**: x86_64 first-class; no macOS/Windows targets, no Scoop/winget.
- systemd unit is a **user** unit (`systemctl --user`), installed to `usr/lib/systemd/user/`. Joséphine runs in the user session, never as root.
- The binary is `josephine`; license `MIT OR Apache-2.0`; maintainer `Kevin Delfour <k@levilainpetit.dev>`; repo `https://github.com/systm-d/josephine`.
- Foreground watcher must be reachable via a stable public command `josephine daemon run` (used by systemd) in addition to the existing `daemon start` (background) — do not break `daemon start`.
- `Cargo.lock` must be committed (binary project → reproducible `--locked` builds).
- Release runs on tags matching `v*.*.*`. crates.io publish order: `josephine-core` then `josephine`.
- Existing gate stays green: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- release.yml and the formulas with release-artifact SHAs are only fully verifiable after an actual tagged release; mark placeholders clearly.
- Branch: `chore/template-alignment-p3-6`.

---

### Task 1: Commit Cargo.lock for reproducible builds

**Files:**
- Modify: `.gitignore` (remove the `Cargo.lock` ignore line)
- Add to git: `Cargo.lock`

- [ ] **Step 1: Stop ignoring Cargo.lock**

Remove the line `Cargo.lock` from `.gitignore` (leave `/target/` and `**/*.rs.bk`).

- [ ] **Step 2: Generate and stage the lockfile**

Run:
```bash
cargo build --workspace   # ensures Cargo.lock is up to date
git add .gitignore Cargo.lock
git status --short          # Cargo.lock should now be staged, not ignored
```
Expected: `Cargo.lock` is staged.

- [ ] **Step 3: Commit**

```bash
git commit -m "build: commit Cargo.lock for reproducible --locked builds"
```

---

### Task 2: Public `josephine daemon run` foreground subcommand

**Files:**
- Modify: `crates/josephine/src/commands/daemon_cmd.rs`
- Modify: `crates/josephine/tests/cli.rs` (add a help-lists-run test)

**Interfaces:**
- Consumes: `josephine_core::daemon::run_daemon_foreground()` (already public, `async`, runs the scheduler in the foreground).
- Produces: `josephine daemon run` — blocking foreground watcher for systemd.

- [ ] **Step 1: Write the failing test in `crates/josephine/tests/cli.rs`**

Add (do NOT invoke `daemon run` itself — it blocks forever; only check `--help` lists it):

```rust
#[test]
fn daemon_help_lists_run() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(contains("run"));
}
```

- [ ] **Step 2: Run it to confirm it fails**

Run: `cargo test -p josephine --test cli daemon_help_lists_run`
Expected: FAIL (no `run` subcommand yet).

- [ ] **Step 3: Add the `Run` variant and handler in `crates/josephine/src/commands/daemon_cmd.rs`**

Add the variant to the enum (after `Logs`):

```rust
    /// Affiche les derniers logs
    Logs,
    /// Exécute le watcher en avant-plan (utilisé par systemd `--user`)
    Run,
```

Add the match arm in `run()` (after the `Logs` arm):

```rust
        DaemonAction::Logs => {
            let logs = control.logs(50)?;
            println!("{logs}");
        }
        DaemonAction::Run => {
            josephine_core::daemon::run_daemon_foreground().await?;
        }
```

- [ ] **Step 4: Confirm the test passes and the gate is green**

Run:
```bash
cargo test -p josephine --test cli daemon_help_lists_run
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```
Expected: the new test passes; full gate green.

- [ ] **Step 5: Commit**

```bash
git add crates/josephine/src/commands/daemon_cmd.rs crates/josephine/tests/cli.rs
git commit -m "feat: add 'daemon run' foreground subcommand for systemd"
```

---

### Task 3: systemd user unit

**Files:**
- Create: `packaging/systemd/josephine.service`

- [ ] **Step 1: Create `packaging/systemd/josephine.service`**

```ini
[Unit]
Description=Joséphine — your computer's guardian angel
Documentation=https://github.com/systm-d/josephine
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/josephine daemon run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

- [ ] **Step 2: Validate the unit file syntax (if systemd tooling is available)**

Run:
```bash
command -v systemd-analyze >/dev/null 2>&1 && systemd-analyze verify --user packaging/systemd/josephine.service || echo "systemd-analyze unavailable; unit will be validated on a real install"
```
Expected: no errors, or the "unavailable" note. (A "directory not found"/ExecStart path note is acceptable since the binary isn't installed at `/usr/bin` in this checkout.)

- [ ] **Step 3: Commit**

```bash
git add packaging/systemd/josephine.service
git commit -m "feat: add systemd user unit for the background watcher"
```

---

### Task 4: deb + rpm Cargo metadata

**Files:**
- Modify: `crates/josephine/Cargo.toml`

- [ ] **Step 1: Append packaging metadata to `crates/josephine/Cargo.toml`**

```toml
[package.metadata.deb]
maintainer = "Kevin Delfour <k@levilainpetit.dev>"
copyright = "2026, Kevin Delfour"
license-file = ["LICENSE-MIT", "0"]
extended-description = "Joséphine watches your Linux machine and warns you, kindly, before something goes wrong. Local-only, no cloud."
section = "utils"
priority = "optional"
assets = [
    ["target/release/josephine", "usr/bin/", "755"],
    ["packaging/systemd/josephine.service", "usr/lib/systemd/user/josephine.service", "644"],
    ["README.md", "usr/share/doc/josephine/README.md", "644"],
    ["LICENSE-MIT", "usr/share/doc/josephine/LICENSE-MIT", "644"],
    ["LICENSE-APACHE", "usr/share/doc/josephine/LICENSE-APACHE", "644"],
]

[package.metadata.generate-rpm]
assets = [
    { source = "target/release/josephine", dest = "/usr/bin/josephine", mode = "755" },
    { source = "packaging/systemd/josephine.service", dest = "/usr/lib/systemd/user/josephine.service", mode = "644" },
    { source = "LICENSE-MIT", dest = "/usr/share/licenses/josephine/LICENSE-MIT", mode = "644" },
    { source = "LICENSE-APACHE", dest = "/usr/share/licenses/josephine/LICENSE-APACHE", mode = "644" },
]
```

- [ ] **Step 2: Verify the manifest still parses and (optionally) the tools accept it**

Run:
```bash
cargo build --workspace
if command -v cargo-deb >/dev/null 2>&1; then cargo build --release -p josephine && cargo deb -p josephine --no-build && ls target/debian/*.deb; else echo "cargo-deb not installed; metadata will be exercised by release.yml"; fi
```
Expected: build passes; if cargo-deb present, a `.deb` is produced.

- [ ] **Step 3: Commit**

```bash
git add crates/josephine/Cargo.toml
git commit -m "build: add deb and rpm packaging metadata"
```

---

### Task 5: AUR PKGBUILD + Homebrew formula

**Files:**
- Create: `packaging/aur/PKGBUILD`
- Create: `packaging/homebrew/josephine.rb`

- [ ] **Step 1: Create `packaging/aur/PKGBUILD`**

```bash
# Maintainer: Kevin Delfour <k@levilainpetit.dev>
pkgname=josephine
pkgver=0.1.0
pkgrel=1
pkgdesc="Your computer's guardian angel — a local Linux system watcher"
arch=('x86_64')
url="https://github.com/systm-d/josephine"
license=('MIT' 'Apache-2.0')
depends=('gcc-libs')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/systm-d/josephine/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo build --frozen --release
}

check() {
  cd "$pkgname-$pkgver"
  cargo test --frozen --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/josephine" "$pkgdir/usr/bin/josephine"
  install -Dm644 "packaging/systemd/josephine.service" "$pkgdir/usr/lib/systemd/user/josephine.service"
  install -Dm644 "LICENSE-MIT" "$pkgdir/usr/share/licenses/$pkgname/LICENSE-MIT"
  install -Dm644 "LICENSE-APACHE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE-APACHE"
}
```

- [ ] **Step 2: Create `packaging/homebrew/josephine.rb`** (Homebrew on Linux; the release `sha256` is filled in by `release.yml` on tag)

```ruby
class Josephine < Formula
  desc "Your computer's guardian angel — a local Linux system watcher"
  homepage "https://github.com/systm-d/josephine"
  url "https://github.com/systm-d/josephine/archive/refs/tags/v0.1.0.tar.gz"
  # Placeholder: release.yml replaces this with the real tarball checksum on tag.
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT OR Apache-2.0"
  head "https://github.com/systm-d/josephine.git", branch: "main"

  depends_on "rust" => :build
  depends_on :linux

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "josephine", shell_output("#{bin}/josephine --version")
  end
end
```

- [ ] **Step 3: Validate shell/ruby syntax where possible**

Run:
```bash
bash -n packaging/aur/PKGBUILD && echo "PKGBUILD shell-syntax ok"
command -v ruby >/dev/null 2>&1 && ruby -c packaging/homebrew/josephine.rb || echo "ruby unavailable; formula validated by brew on submission"
```
Expected: PKGBUILD parses; formula `Syntax OK` if ruby present.

- [ ] **Step 4: Commit**

```bash
git add packaging/aur/PKGBUILD packaging/homebrew/josephine.rb
git commit -m "feat: add AUR PKGBUILD and Homebrew (Linux) formula"
```

---

### Task 6: Release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags: ["v*.*.*"]

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  artifacts:
    name: build artifacts (linux x86_64)
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build release binary
        run: cargo build --release --locked -p josephine
      - name: Stage tarball + checksum
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          DIST="josephine-${VERSION}-x86_64-unknown-linux-gnu"
          mkdir -p "$DIST"
          cp target/release/josephine README.md CHANGELOG.md LICENSE-MIT LICENSE-APACHE "$DIST"/
          cp -r packaging "$DIST"/
          tar czf "$DIST.tar.gz" "$DIST"
          sha256sum "$DIST.tar.gz" | tee "$DIST.tar.gz.sha256"
      - name: Build .deb and .rpm
        run: |
          cargo install cargo-deb cargo-generate-rpm
          cargo deb -p josephine --no-build
          cargo generate-rpm -p crates/josephine
      - name: Publish GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            josephine-*-x86_64-unknown-linux-gnu.tar.gz
            josephine-*-x86_64-unknown-linux-gnu.tar.gz.sha256
            target/debian/*.deb
            target/generate-rpm/*.rpm

  crates-io:
    name: publish to crates.io
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish josephine-core then josephine
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo publish -p josephine-core --locked
          cargo publish -p josephine --locked
```

- [ ] **Step 2: Validate the workflow YAML**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('yaml ok')"
command -v actionlint >/dev/null 2>&1 && actionlint .github/workflows/release.yml || echo "actionlint unavailable; validated by GitHub on tag"
```
Expected: `yaml ok`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add tag-driven release workflow (GitHub Release + crates.io + deb/rpm)"
```

---

### Task 7: CHANGELOG note

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add bullets under `[Unreleased]` → `### Added`**

```markdown
- `josephine daemon run` foreground subcommand for systemd supervision.
- Packaging: systemd user unit, deb/rpm metadata, AUR PKGBUILD, Homebrew (Linux)
  formula, and a tag-driven release workflow (GitHub Releases + crates.io).
- Committed `Cargo.lock` for reproducible builds.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: note packaging & release in CHANGELOG"
```

---

## Done criteria for Phase 5

- `Cargo.lock` committed; `.gitignore` no longer ignores it.
- `josephine daemon run` exists (test passes); `daemon start` still works.
- `packaging/systemd/josephine.service` (user unit), `packaging/aur/PKGBUILD`, `packaging/homebrew/josephine.rb` present.
- `crates/josephine/Cargo.toml` has `[package.metadata.deb]` + `[package.metadata.generate-rpm]`.
- `.github/workflows/release.yml` present; YAML valid.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass.

## Notes / GitHub-side verification (after push / tag)

- `release.yml` and the deb/rpm/AUR/Homebrew artifacts are only fully validated by an actual `vX.Y.Z` tag. crates.io publish needs a `CARGO_REGISTRY_TOKEN` secret and the `josephine`/`josephine-core` names to be available.
- The Homebrew formula `sha256` is a placeholder until the first release tarball exists (or is updated by an automation step later).
- ARM64 (aarch64) Linux builds are intentionally out of this first cut; add a cross-build matrix entry later if needed.
- A future improvement: route the daemon's logs to the systemd journal (stdout) when run under the unit, instead of the app log file.
