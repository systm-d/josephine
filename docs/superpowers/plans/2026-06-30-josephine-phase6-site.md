# Joséphine — Phase 6: Zola Site + GitHub Pages Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a small, self-contained Zola static site (landing page with hero, features, install, usage) styled with the project brand color, and a GitHub Pages workflow that builds and deploys it on pushes to `main`.

**Architecture:** A `site/` directory holding a minimal custom Zola theme (Tera templates + a single SCSS file compiled by Zola) and one content section (`content/_index.md`). No external Zola theme dependency — the templates live in-repo so the look is fully controlled. `.github/workflows/pages.yml` installs Zola, builds `site/`, and deploys via `actions/deploy-pages` as a project page. Rust API docs stay on docs.rs (not duplicated).

**Tech Stack:** Zola (static site generator), Tera templates, SCSS, GitHub Pages (`actions/deploy-pages`).

## Global Constraints

- Brand color: **`#E0A458`** (amber/gold — the "guardian angel" theme).
- Site language: **English** (docs standard). It may mention that the app's user-facing strings are French.
- `base_url`: `https://systm-d.github.io/josephine` (GitHub **project** page). Adjustable if a custom domain is set later.
- Repo URL referenced on the site: `https://github.com/systm-d/josephine`.
- No Node toolchain — Zola is a single binary.
- Self-contained: no remote Zola theme; templates + SCSS live under `site/`.
- Does not touch Rust code or other workflows. The site build is verifiable locally with `zola build` (if installed); the Pages deploy is only verifiable after push + enabling Pages (Source = GitHub Actions) in repo settings.
- Branch: `chore/template-alignment-p3-6`.

---

### Task 1: Zola site scaffold (config, theme, content)

**Files:**
- Create: `site/config.toml`
- Create: `site/sass/main.scss`
- Create: `site/templates/base.html`
- Create: `site/templates/index.html`
- Create: `site/content/_index.md`

**Interfaces:**
- Produces: a buildable Zola site whose homepage renders the landing content.

- [ ] **Step 1: Create `site/config.toml`**

```toml
base_url = "https://systm-d.github.io/josephine"
title = "Joséphine"
description = "Your computer's guardian angel"
default_language = "en"
compile_sass = true
build_search_index = false

[markdown]
highlight_code = true

[extra]
brand_color = "#E0A458"
repo_url = "https://github.com/systm-d/josephine"
```

- [ ] **Step 2: Create `site/sass/main.scss`** (compiled to `main.css` at the site root)

```scss
$brand: #e0a458;

* { box-sizing: border-box; }

body {
  margin: 0;
  font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  line-height: 1.6;
  color: #2b2b2b;
  background: #fffdf8;
}

a { color: $brand; }

.hero {
  text-align: center;
  padding: 5rem 1rem 3rem;
  background: linear-gradient(180deg, rgba(224, 164, 88, 0.14), transparent);
}
.hero h1 { font-size: 3rem; margin: 0; }
.hero .tagline { font-size: 1.3rem; color: #6b6b6b; margin-top: .5rem; }

.container { max-width: 760px; margin: 0 auto; padding: 0 1.25rem 4rem; }

.button {
  display: inline-block;
  padding: .7rem 1.4rem;
  border-radius: 8px;
  background: $brand;
  color: #fff;
  text-decoration: none;
  font-weight: 600;
}

.features {
  display: grid;
  gap: 1.5rem;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  margin: 3rem 0;
}
.feature h3 { margin-bottom: .3rem; }

pre {
  background: #2b2b2b;
  color: #f5f5f5;
  padding: 1rem;
  border-radius: 8px;
  overflow-x: auto;
}
code { font-family: ui-monospace, "SFMono-Regular", Menlo, monospace; }

footer { text-align: center; padding: 2rem; color: #9a9a9a; font-size: .9rem; }
```

- [ ] **Step 3: Create `site/templates/base.html`**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{% block title %}{{ config.title }}{% endblock title %}</title>
    <meta name="description" content="{{ config.description }}" />
    <link rel="stylesheet" href="{{ get_url(path='main.css') }}" />
  </head>
  <body>
    {% block content %}{% endblock content %}
    <footer>
      Joséphine — local, Linux, open source.
      <a href="{{ config.extra.repo_url }}">GitHub</a>
    </footer>
  </body>
</html>
```

- [ ] **Step 4: Create `site/templates/index.html`** (Zola renders the root section with this)

```html
{% extends "base.html" %}
{% block content %}
<header class="hero">
  <h1>✨ {{ config.title }}</h1>
  <p class="tagline">{{ config.description }}</p>
  <p><a class="button" href="{{ config.extra.repo_url }}">View on GitHub</a></p>
</header>
<main class="container">
  {{ section.content | safe }}
</main>
{% endblock content %}
```

- [ ] **Step 5: Create `site/content/_index.md`**

```markdown
+++
title = "Joséphine"
+++

Joséphine watches your Linux machine silently and only speaks up when it helps —
keeping an eye on CPU, memory, disk, temperature and systemd services, and sending
warm, plain-language desktop notifications. 100% local; no data ever leaves your
computer.

<div class="features">
  <div class="feature">
    <h3>🩺 Five checks</h3>
    <p>CPU, memory, disk, temperature, systemd — early warnings before trouble.</p>
  </div>
  <div class="feature">
    <h3>🔔 Kind notifications</h3>
    <p>Warm desktop messages, never alarmist — never ERROR/FATAL/PANIC.</p>
  </div>
  <div class="feature">
    <h3>🔒 Local &amp; private</h3>
    <p>Everything runs on your machine. No cloud, no telemetry.</p>
  </div>
</div>

## Install

```sh
cargo install josephine
```

## Usage

```sh
josephine            # quick status
josephine doctor     # detailed diagnostics
josephine daemon start
```

User-facing messages are intentionally in French — that is part of Joséphine's
character.
```

- [ ] **Step 6: Build the site to verify (if Zola is installed)**

Run:
```bash
if command -v zola >/dev/null 2>&1; then (cd site && zola build) && test -f site/public/index.html && echo "zola build ok"; else echo "zola not installed; the Pages workflow will build it on push"; fi
```
Expected: `zola build ok` (and `site/public/index.html` exists), or the not-installed note. Do NOT commit `site/public/` (add it to gitignore in Step 7).

- [ ] **Step 7: Ignore the Zola build output and commit**

Append to `.gitignore`:

```
/site/public/
```

Then:
```bash
git add site/config.toml site/sass site/templates site/content .gitignore
git commit -m "feat: add Zola landing site (brand color, hero, features, install)"
```

---

### Task 2: GitHub Pages workflow

**Files:**
- Create: `.github/workflows/pages.yml`

- [ ] **Step 1: Create `.github/workflows/pages.yml`**

```yaml
name: Pages

on:
  push:
    branches: [main]
    paths:
      - "site/**"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - name: Install Zola
        run: |
          ZOLA_VERSION=0.19.2
          curl -sSL "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar xz
          sudo mv zola /usr/local/bin/
      - name: Build site
        run: cd site && zola build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: site/public

  deploy:
    needs: build
    runs-on: ubuntu-24.04
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Validate the workflow YAML**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/pages.yml')); print('yaml ok')"
command -v actionlint >/dev/null 2>&1 && actionlint .github/workflows/pages.yml || echo "actionlint unavailable; validated by GitHub on push"
```
Expected: `yaml ok`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/pages.yml
git commit -m "ci: add GitHub Pages workflow (Zola build + deploy-pages)"
```

---

### Task 3: CHANGELOG note

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add a bullet under `[Unreleased]` → `### Added`**

```markdown
- Project landing site (Zola) deployed to GitHub Pages.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: note the landing site in CHANGELOG"
```

---

## Done criteria for Phase 6

- `site/` contains `config.toml`, `sass/main.scss`, `templates/{base,index}.html`, `content/_index.md`.
- `cd site && zola build` succeeds (if Zola available locally) and produces `site/public/index.html`.
- `site/public/` is gitignored (not committed).
- `.github/workflows/pages.yml` present; YAML valid.
- CHANGELOG `[Unreleased]` notes the site.
- No Rust code changed; `cargo build --workspace` still passes.

## Notes / GitHub-side verification (after push)

- GitHub Pages must be enabled with **Source = GitHub Actions** in repo settings for the deploy to publish; the build job runs regardless.
- `base_url` assumes the project page `https://systm-d.github.io/josephine`; change it if a custom domain or org page is used.
- Pin/refresh the `ZOLA_VERSION` (0.19.2) as Zola releases progress.
