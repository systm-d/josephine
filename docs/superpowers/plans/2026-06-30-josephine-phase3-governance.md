# Joséphine — Phase 3: Governance & Supply-Chain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the open-source governance and supply-chain files the `rust-cli-template` mood requires — contributor docs, code of conduct, security policy, shared conventions, an AI/contributor dev guide, GitHub community templates, Dependabot, CODEOWNERS, and a `cargo-deny` policy — and finish the deferred `josephine-cli` doc cleanup.

**Architecture:** All additions are documentation and configuration; no Rust source changes (the build/tests stay green throughout, used only as a regression guard). `CONVENTIONS.md` is the single written source of truth for shared standards; `CLAUDE.md` becomes the canonical project dev guide and `AGENTS.md` is reduced to a one-line pointer to it (DRY); other governance files reference `CONVENTIONS.md` rather than restating standards. `.github/` holds GitHub-native community health files. `deny.toml` encodes the supply-chain license/advisory policy that Phase 4 CI will enforce.

**Tech Stack:** Markdown, GitHub issue-forms YAML, Dependabot YAML, TOML (`deny.toml` for `cargo-deny`). No new Rust dependencies.

## Global Constraints

- **Docs language:** English for all governance docs (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CONVENTIONS, CLAUDE.md, GitHub templates). User-facing CLI/notification strings stay French (do not touch them here).
- **Security/contact:** `k@levilainpetit.dev`.
- **Repository:** `https://github.com/systm-d/josephine`. GitHub owner handle for CODEOWNERS/reviewers: `@kdelfour` — **verify this is the correct GitHub username before relying on it** (the repo lives under the `systm-d` org; adjust to a team like `@systm-d/maintainers` if that is preferred).
- **Crate/paths:** binary crate is `josephine` (dir `crates/josephine/`), library is `josephine-core` (dir `crates/josephine-core/`). No remaining `josephine-cli` path/command references may exist after this phase except the intentional historical mention in `CHANGELOG.md`.
- **Standards referenced (defined elsewhere, restated only in CONVENTIONS.md):** edition 2024, MSRV 1.85, rustfmt `max_width = 100`, `unsafe_code = "forbid"`, clippy `all = warn`, dual license `MIT OR Apache-2.0`, Conventional Commits, Keep a Changelog + SemVer, module discipline (`josephine-core` = pure logic, `josephine` = thin CLI dispatch), **Linux-only** target (systemd / `/sys/class/thermal` / libnotify).
- **No source changes:** do not modify any `.rs` file. After each task, `cargo build --workspace` and `cargo test --workspace` must still pass (regression guard).
- Reference template: `/home/kdelfour/Workspace/Professionel/_templates/cli/` (note: it does not yet realize any governance files, so these are authored from this plan).
- Work on a dedicated branch off `main` (e.g. `chore/phase3-governance`).

---

### Task 1: CONVENTIONS.md — shared standards source of truth

**Files:**
- Create: `CONVENTIONS.md`

**Interfaces:**
- Produces: `CONVENTIONS.md` at repo root — later governance files link to it instead of restating standards.

- [ ] **Step 1: Create `CONVENTIONS.md`**

```markdown
# Conventions

The written source of truth for the standards shared across this project (and the
other CLIs built from the same `rust-cli-template` mood). Governance files link here
instead of restating these rules.

## Language & Edition

- Rust **edition 2024**, MSRV **1.85** (pinned via `rust-toolchain.toml`).
- Formatting: `rustfmt` with `max_width = 100`, edition 2024 (`rustfmt.toml`).
  `cargo fmt --check` must pass.
- Lints: `unsafe_code = "forbid"`; clippy `all = { level = "warn", priority = -1 }`,
  inherited per crate via `[lints] workspace = true`.
  `cargo clippy --workspace --all-targets -- -D warnings` must pass.

## Project shape

- Workspace: `josephine-core` (pure logic library) + `josephine` (binary, thin CLI).
- Module discipline: business logic lives in `josephine-core`; argument parsing,
  dispatch, and output formatting live in the `josephine` binary (`cli.rs`,
  `commands/`, `output/`). The CLI dispatch is thin — no business logic in `cli.rs`.
- **Linux-only** by design: checks rely on systemd (`systemctl`),
  `/sys/class/thermal`, and desktop notifications via libnotify.

## Language of text

- Documentation (README, this file, governance, future site) is in **English**.
- User-facing strings — CLI output and desktop notifications — are in **French**
  and intentionally warm (Joséphine, the guardian angel). Never `ERROR`/`FATAL`/
  `PANIC` in user-facing text.
- Code identifiers are in English.

## Git & releases

- **Conventional Commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`,
  `test:`, `build:`, `style:`, `ci:`).
- **Keep a Changelog** format in `CHANGELOG.md`; **Semantic Versioning**.
- Dual license: **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`).

## Database

- SQLite via `rusqlite`. Schema changes are versioned migrations in
  `crates/josephine-core/migrations/` (`V00N__description.sql`), applied through the
  `schema_version` table. Multi-statement migrations must be wrapped in a transaction.

## Quality gate (run before every PR)

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
```

- [ ] **Step 2: Verify and commit**

Run: `test -f CONVENTIONS.md && cargo build --workspace`
Expected: file exists; build passes (no source changed).

```bash
git add CONVENTIONS.md
git commit -m "docs: add CONVENTIONS.md (shared standards source of truth)"
```

---

### Task 2: Governance trio — CONTRIBUTING, CODE_OF_CONDUCT, SECURITY

**Files:**
- Create: `CONTRIBUTING.md`
- Create: `CODE_OF_CONDUCT.md`
- Create: `SECURITY.md`

**Interfaces:**
- Consumes: `CONVENTIONS.md` (linked, not restated).
- Produces: the three standard community health docs at repo root.

- [ ] **Step 1: Create `CONTRIBUTING.md`**

```markdown
# Contributing to Joséphine

Thanks for your interest in improving Joséphine — your computer's guardian angel.

## Before you start

- Read the [conventions](CONVENTIONS.md): edition, formatting, lints, commit style.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
- Joséphine is **Linux-only** (it relies on systemd, `/sys/class/thermal`, and
  libnotify). Changes should keep that target in mind.

## Development setup

```sh
git clone https://github.com/systm-d/josephine
cd josephine
cargo build
```

The toolchain is pinned by `rust-toolchain.toml` (stable + rustfmt + clippy).

## Quality gate

Run this before opening a pull request — CI enforces the same:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Commits & pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, …).
- Add a `CHANGELOG.md` entry under `[Unreleased]` for user-visible changes.
- Keep user-facing strings (CLI output, notifications) in French and warm; keep
  docs and code identifiers in English.
- One focused change per PR. Fill in the pull request template.

## Reporting bugs & ideas

Open an issue using the bug or feature template. For security issues, do **not**
open a public issue — see [SECURITY.md](SECURITY.md).
```

- [ ] **Step 2: Create `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, verbatim with our contact)**

```markdown
# Contributor Covenant Code of Conduct

## Our Pledge

We as members, contributors, and leaders pledge to make participation in our
community a harassment-free experience for everyone, regardless of age, body
size, visible or invisible disability, ethnicity, sex characteristics, gender
identity and expression, level of experience, education, socio-economic status,
nationality, personal appearance, race, caste, color, religion, or sexual
identity and orientation.

We pledge to act and interact in ways that contribute to an open, welcoming,
diverse, inclusive, and healthy community.

## Our Standards

Examples of behavior that contributes to a positive environment for our
community include:

* Demonstrating empathy and kindness toward other people
* Being respectful of differing opinions, viewpoints, and experiences
* Giving and gracefully accepting constructive feedback
* Accepting responsibility and apologizing to those affected by our mistakes,
  and learning from the experience
* Focusing on what is best not just for us as individuals, but for the overall
  community

Examples of unacceptable behavior include:

* The use of sexualized language or imagery, and sexual attention or advances of
  any kind
* Trolling, insulting or derogatory comments, and personal or political attacks
* Public or private harassment
* Publishing others' private information, such as a physical or email address,
  without their explicit permission
* Other conduct which could reasonably be considered inappropriate in a
  professional setting

## Enforcement Responsibilities

Community leaders are responsible for clarifying and enforcing our standards of
acceptable behavior and will take appropriate and fair corrective action in
response to any behavior that they deem inappropriate, threatening, offensive,
or harmful.

Community leaders have the right and responsibility to remove, edit, or reject
comments, commits, code, wiki edits, issues, and other contributions that are
not aligned to this Code of Conduct, and will communicate reasons for moderation
decisions when appropriate.

## Scope

This Code of Conduct applies within all community spaces, and also applies when
an individual is officially representing the community in public spaces.
Examples of representing our community include using an official email address,
posting via an official social media account, or acting as an appointed
representative at an online or offline event.

## Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be
reported to the community leaders responsible for enforcement at
k@levilainpetit.dev.
All complaints will be reviewed and investigated promptly and fairly.

All community leaders are obligated to respect the privacy and security of the
reporter of any incident.

## Enforcement Guidelines

Community leaders will follow these Community Impact Guidelines in determining
the consequences for any action they deem in violation of this Code of Conduct:

### 1. Correction

**Community Impact**: Use of inappropriate language or other behavior deemed
unprofessional or unwelcome in the community.

**Consequence**: A private, written warning from community leaders, providing
clarity around the nature of the violation and an explanation of why the
behavior was inappropriate. A public apology may be requested.

### 2. Warning

**Community Impact**: A violation through a single incident or series of
actions.

**Consequence**: A warning with consequences for continued behavior. No
interaction with the people involved, including unsolicited interaction with
those enforcing the Code of Conduct, for a specified period of time. This
includes avoiding interactions in community spaces as well as external channels
like social media. Violating these terms may lead to a temporary or permanent
ban.

### 3. Temporary Ban

**Community Impact**: A serious violation of community standards, including
sustained inappropriate behavior.

**Consequence**: A temporary ban from any sort of interaction or public
communication with the community for a specified period of time. No public or
private interaction with the people involved, including unsolicited interaction
with those enforcing the Code of Conduct, is allowed during this period.
Violating these terms may lead to a permanent ban.

### 4. Permanent Ban

**Community Impact**: Demonstrating a pattern of violation of community
standards, including sustained inappropriate behavior, harassment of an
individual, or aggression toward or disparagement of classes of individuals.

**Consequence**: A permanent ban from any sort of public interaction within the
community.

## Attribution

This Code of Conduct is adapted from the [Contributor Covenant][homepage],
version 2.1, available at
[https://www.contributor-covenant.org/version/2/1/code_of_conduct.html][v2.1].

Community Impact Guidelines were inspired by
[Mozilla's code of conduct enforcement ladder][Mozilla CoC].

For answers to common questions about this code of conduct, see the FAQ at
[https://www.contributor-covenant.org/faq][FAQ]. Translations are available at
[https://www.contributor-covenant.org/translations][translations].

[homepage]: https://www.contributor-covenant.org
[v2.1]: https://www.contributor-covenant.org/version/2/1/code_of_conduct.html
[Mozilla CoC]: https://github.com/mozilla/diversity
[FAQ]: https://www.contributor-covenant.org/faq
[translations]: https://www.contributor-covenant.org/translations
```

- [ ] **Step 3: Create `SECURITY.md`**

```markdown
# Security Policy

## Supported versions

Joséphine is pre-1.0; security fixes target the latest released version and `main`.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |

## Reporting a vulnerability

Please report security issues **privately** — do not open a public issue.

- Email: **k@levilainpetit.dev**
- Or use GitHub's private vulnerability reporting:
  <https://github.com/systm-d/josephine/security/advisories/new>

Include a description, reproduction steps, and the affected version. We aim to
acknowledge reports within 7 days and to coordinate a fix and disclosure
timeline with you.

Joséphine runs entirely locally and sends no data over the network, which limits
the attack surface, but we take any report seriously.
```

- [ ] **Step 4: Verify and commit**

Run: `test -f CONTRIBUTING.md && test -f CODE_OF_CONDUCT.md && test -f SECURITY.md && grep -q "k@levilainpetit.dev" CODE_OF_CONDUCT.md SECURITY.md && cargo build --workspace`
Expected: all files exist, contact present, build passes.

```bash
git add CONTRIBUTING.md CODE_OF_CONDUCT.md SECURITY.md
git commit -m "docs: add CONTRIBUTING, CODE_OF_CONDUCT (Covenant 2.1), SECURITY"
```

---

### Task 3: CLAUDE.md dev guide + AGENTS.md pointer + finish josephine-cli doc cleanup

**Files:**
- Create: `CLAUDE.md`
- Modify: `AGENTS.md` (replace contents with a short pointer)
- Modify: `docs/ARCHITECTURE.md` (line 11: `josephine-cli` → `josephine`)
- Modify: `docs/CURRENT_STATE.md` (lines 63, 144: `josephine-cli` → `josephine`)

**Interfaces:**
- Consumes: `CONVENTIONS.md`, `CONTRIBUTING.md` (linked).
- Produces: `CLAUDE.md` canonical project guide; `AGENTS.md` reduced to a pointer.

- [ ] **Step 1: Create `CLAUDE.md` (canonical project dev guide, updated for the `josephine` rename)**

```markdown
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
```

- [ ] **Step 2: Replace `AGENTS.md` contents with a pointer (DRY — single source is CLAUDE.md)**

Overwrite `AGENTS.md` with exactly:

```markdown
# Joséphine — agent guide

This project's agent/contributor guide is **[CLAUDE.md](CLAUDE.md)**. Start there.

See also [CONVENTIONS.md](CONVENTIONS.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
```

- [ ] **Step 3: Fix the stale `josephine-cli` references in the two docs**

In `docs/ARCHITECTURE.md` line 11 and `docs/CURRENT_STATE.md` lines 63 and 144, replace `josephine-cli` with `josephine`. A safe, targeted way (only the `josephine-cli` token changes; `josephine-core` does not contain that substring):

```bash
sed -i 's/josephine-cli/josephine/g' docs/ARCHITECTURE.md docs/CURRENT_STATE.md
```

- [ ] **Step 4: Verify no stray references remain (except the intentional CHANGELOG mention) and commit**

Run:
```bash
git grep -n "josephine-cli" -- . ':!CHANGELOG.md' ':!docs/superpowers'
```
Expected: NO output (the only remaining `josephine-cli` mentions are the intentional one in `CHANGELOG.md` and historical ones inside `docs/superpowers/` plan/spec records, which we leave as written history).
Then: `test -f CLAUDE.md && cargo build --workspace`
Expected: passes.

```bash
git add CLAUDE.md AGENTS.md docs/ARCHITECTURE.md docs/CURRENT_STATE.md
git commit -m "docs: add CLAUDE.md guide, reduce AGENTS.md to pointer, finish josephine-cli cleanup"
```

---

### Task 4: GitHub community templates (issues + PR)

**Files:**
- Create: `.github/ISSUE_TEMPLATE/bug.yml`
- Create: `.github/ISSUE_TEMPLATE/feature.yml`
- Create: `.github/ISSUE_TEMPLATE/config.yml`
- Create: `.github/PULL_REQUEST_TEMPLATE.md`

**Interfaces:**
- Produces: GitHub-native issue forms and PR template.

- [ ] **Step 1: Create `.github/ISSUE_TEMPLATE/bug.yml`**

```yaml
name: Bug report
description: Something isn't working as expected
labels: ["bug"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug. Joséphine is Linux-only.
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: What did you expect, and what happened instead?
    validations:
      required: true
  - type: textarea
    id: repro
    attributes:
      label: Steps to reproduce
      placeholder: |
        1. Run `josephine ...`
        2. ...
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Joséphine version
      description: Output of `josephine --version`
    validations:
      required: true
  - type: input
    id: distro
    attributes:
      label: Linux distribution & version
      placeholder: e.g. Fedora 41, Ubuntu 24.04
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: Relevant logs
      description: Output of `josephine doctor` or `josephine daemon logs`, if relevant
      render: shell
    validations:
      required: false
```

- [ ] **Step 2: Create `.github/ISSUE_TEMPLATE/feature.yml`**

```yaml
name: Feature request
description: Suggest an idea for Joséphine
labels: ["enhancement"]
body:
  - type: textarea
    id: problem
    attributes:
      label: Problem
      description: What problem would this solve? What are you trying to do?
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposed solution
      description: What would you like Joséphine to do?
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
    validations:
      required: false
```

- [ ] **Step 3: Create `.github/ISSUE_TEMPLATE/config.yml`**

```yaml
blank_issues_enabled: false
contact_links:
  - name: Security report
    url: https://github.com/systm-d/josephine/security/advisories/new
    about: Please report security vulnerabilities privately, not as public issues.
```

- [ ] **Step 4: Create `.github/PULL_REQUEST_TEMPLATE.md`**

```markdown
## Summary

<!-- What does this change and why? -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Refactor / chore

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `CHANGELOG.md` updated under `[Unreleased]` (for user-visible changes)
- [ ] Commits follow Conventional Commits
- [ ] User-facing strings stay in French; docs/identifiers in English
```

- [ ] **Step 5: Validate YAML well-formedness and commit**

Run:
```bash
python3 -c "import yaml,sys; [yaml.safe_load(open(f)) for f in ['.github/ISSUE_TEMPLATE/bug.yml','.github/ISSUE_TEMPLATE/feature.yml','.github/ISSUE_TEMPLATE/config.yml']]; print('yaml ok')"
```
Expected: prints `yaml ok` (no parse error). If `python3`/`yaml` is unavailable, instead run `ruby -ryaml -e "%w[.github/ISSUE_TEMPLATE/bug.yml .github/ISSUE_TEMPLATE/feature.yml .github/ISSUE_TEMPLATE/config.yml].each{|f| YAML.load_file(f)}; puts 'yaml ok'"`.

```bash
git add .github/ISSUE_TEMPLATE/ .github/PULL_REQUEST_TEMPLATE.md
git commit -m "ci: add GitHub issue forms and pull request template"
```

---

### Task 5: Dependabot + CODEOWNERS

**Files:**
- Create: `.github/dependabot.yml`
- Create: `.github/CODEOWNERS`

**Interfaces:**
- Produces: weekly dependency update config (cargo + github-actions) and code ownership.

- [ ] **Step 1: Create `.github/dependabot.yml`**

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
    commit-message:
      prefix: "build"
  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: weekly
    commit-message:
      prefix: "ci"
```

- [ ] **Step 2: Create `.github/CODEOWNERS`**

```
# Default owner for everything in the repo.
# NOTE: verify @kdelfour is the correct GitHub handle (repo is under the systm-d org;
# switch to a team like @systm-d/maintainers if preferred).
* @kdelfour
```

- [ ] **Step 3: Validate YAML and commit**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml')); print('yaml ok')"
```
Expected: `yaml ok`. (Ruby fallback as in Task 4 if needed.)

```bash
git add .github/dependabot.yml .github/CODEOWNERS
git commit -m "ci: add Dependabot config and CODEOWNERS"
```

---

### Task 6: deny.toml — supply-chain policy (cargo-deny)

**Files:**
- Create: `deny.toml`

**Interfaces:**
- Produces: `deny.toml` policy that Phase 4 CI (`cargo-deny`) will enforce.

- [ ] **Step 1: Create `deny.toml`**

```toml
# cargo-deny configuration — supply-chain policy.
# Run locally with: cargo deny check
# See https://embarkstudios.github.io/cargo-deny/

[advisories]
# Use the default advisory database; fail on any unmaintained/vulnerable crate.
version = 2
yanked = "deny"

[licenses]
version = 2
# Licenses we accept across the dependency tree (project itself is MIT OR Apache-2.0).
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Zlib",
    "MPL-2.0",
    "CC0-1.0",
]
confidence-threshold = 0.9

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

- [ ] **Step 2: Validate the policy**

If `cargo-deny` is installed, run the real check and fix any genuine policy gaps it reports (e.g., a license used by a transitive dependency that is acceptable but missing from `allow`):

```bash
if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny check licenses bans sources advisories
else
  echo "cargo-deny not installed; validating TOML syntax only (CI in Phase 4 will run the full check)"
  python3 -c "import tomllib; tomllib.load(open('deny.toml','rb')); print('toml ok')"
fi
```
Expected: `cargo deny check` passes, OR (if not installed) `toml ok`. If `cargo deny check licenses` reports an allowed-but-unlisted license actually present in the tree, add that exact SPDX id to the `allow` list and re-run. Do **not** weaken `bans`/`sources` to silence findings without cause.

- [ ] **Step 3: Commit**

```bash
git add deny.toml
git commit -m "build: add cargo-deny supply-chain policy (deny.toml)"
```

---

## Done criteria for Phase 3

- Root governance files present: `CONVENTIONS.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Covenant 2.1, contact `k@levilainpetit.dev`), `SECURITY.md`, `CLAUDE.md`.
- `AGENTS.md` reduced to a pointer to `CLAUDE.md`.
- `.github/` has `ISSUE_TEMPLATE/{bug,feature,config}.yml`, `PULL_REQUEST_TEMPLATE.md`, `dependabot.yml`, `CODEOWNERS`.
- `deny.toml` present and (if `cargo-deny` available) passing.
- `git grep "josephine-cli" -- . ':!CHANGELOG.md' ':!docs/superpowers'` returns nothing.
- `cargo build --workspace` and `cargo test --workspace` still pass (no source changed).

## Notes for later phases

- **Phase 4 (CI)** will add `.github/workflows/ci.yml` that runs `cargo-deny`, so the `deny.toml` allow-list may need a top-up once the full dependency tree is checked in CI. The README's CI/crates.io badges (added in Phase 1) stay broken until Phase 4/5 land the workflow and a published crate.
- Verify the `@kdelfour` GitHub handle in `CODEOWNERS` (and adjust Dependabot reviewers if you add any).
```