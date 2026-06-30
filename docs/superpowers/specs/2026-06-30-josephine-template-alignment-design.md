# Design — Aligner Joséphine sur le « mood » `rust-cli-template`

- **Date** : 2026-06-30
- **Statut** : Approuvé (phase de design)
- **Auteur** : Kevin Delfour
- **Objectif** : Réaligner le projet Joséphine (CLI Rust, « l'ange gardien de votre
  ordinateur »), fait rapidement en v0.1, sur le standard défini par le template
  `rust-cli-template`. Joséphine est le **premier projet porté dans ce mood**.

---

## 1. Contexte & motivation

Joséphine est un workspace Rust sain (`josephine-core` + `josephine-cli`) avec une
architecture claire (dispatch clap *thin*, `anyhow::Result`, séparation core/cli) mais sans
l'outillage et les conventions partagées du template : edition 2021, licence MIT simple,
ni `rust-toolchain.toml`, ni `rustfmt.toml`, ni lints, ni profil release, ni CHANGELOG, ni
tests d'intégration CLI, ni CI/packaging/site.

Le template `rust-cli-template` (`/home/kdelfour/Workspace/Professionel/_templates/cli/`)
définit le standard. **Distinction importante** : le template *réalisé* à ce jour ne couvre
que la **fondation** (edition 2024 / MSRV 1.85, toolchain, rustfmt, lints, profil release,
double licence, CHANGELOG, README à badges, squelette `run() -> ExitCode` + `cli.rs` +
`commands/`, tests `assert_cmd`). La **spec complète** du template
(`docs/superpowers/specs/2026-06-27-cli-template-design.md`) décrit en plus CI, packaging,
gouvernance, site Zola, `deny.toml`, coverage tarpaulin, benches — non encore réalisés dans
le template. Cette spec-ci vise le **mood complet** (spec), pas seulement la fondation.

La spec du template désignait *repolens* comme projet pilote. On dévie volontairement :
Joséphine est porté en premier.

### Axes Joséphine (vocabulaire `cargo-generate.toml` du template)

| Axe | Valeur | Conséquence |
|---|---|---|
| `topology` | `workspace` | déjà : `crates/josephine-core` (lib) + `crates/josephine` (bin) |
| `ui` | `cli` | pas de TUI |
| `state` | `sqlite` | déjà : `rusqlite` ; adopter le pattern `migrations/` |
| `privileges` | `single` | pas de helper privilégié |
| `service` | `daemon` | déjà : démon de surveillance ; ajouter une unit systemd |

---

## 2. Principes directeurs

- **Iso-fonctionnel** : à chaque phase, le comportement de l'appli est préservé. On ajoute
  de l'outillage et on aligne les conventions, on ne réécrit pas le métier.
- **Personnalité préservée** : message d'erreur bienveillant
  (`✨ Joséphine a rencontré un souci : …`) et notifications/strings utilisateur **en
  français**. La documentation (README, CHANGELOG, gouvernance, site) passe en **anglais**
  (standard « English-first docs, user-facing strings may be French »).
- **Module discipline** : `josephine-core` = logique pure (pas d'IO d'orchestration CLI) ;
  le crate binaire `josephine` = dispatch *thin* + IO (`cli.rs`, `commands/`, `output/`).
  Décision : on **garde** `cli.rs`/`commands/` dans le binaire (le template les met dans
  `core`, mais c'est une simplification de scaffolding ; la séparation actuelle de Joséphine
  est plus propre et conforme à l'esprit « core = pure logic »).
- **Vérification par phase** : `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo build`, `cargo test` verts avant de passer à la phase suivante.

---

## 3. Décisions confirmées

- **Portée** : mood complet (spec).
- **Langue** : README/docs en anglais ; strings utilisateur (notifications, messages CLI) en
  français.
- **Nommage** : crate binaire `josephine-cli` → `josephine` (le binaire s'appelle déjà
  `josephine`) ; `josephine-core` inchangé.
- **Packaging / matrice** : **Linux-pragmatique** (défaut retenu, confirmable). Joséphine est
  Linux-only (checks `systemd`, lecture `/sys/class/thermal`, notifications `libnotify`). On
  cible donc CI Ubuntu 22.04/24.04 + Fedora 40/41, packaging deb + rpm + AUR +
  Homebrew-Linux, et on **omet** Windows (Scoop/winget) et macOS pour ne pas produire de
  packaging non fonctionnel. Inversable vers le cross-platform complet si souhaité.
- **Métadonnées (défauts)** : repo `https://github.com/systm-d/josephine`, auteur
  `Kevin Delfour`, contact sécurité `k@levilainpetit.dev`, brand color ambre/or `#E0A458`
  (thème « ange ✨ », ajustable).

---

## 4. Plan par phases

### Phase 1 — Standards & parité de code

- `edition = "2024"` + `rust-version = "1.85"` dans `[workspace.package]`.
- `rust-toolchain.toml` (`channel = "stable"`, `components = ["rustfmt", "clippy"]`).
- `rustfmt.toml` (`edition = "2024"`, `max_width = 100`) + reformatage one-shot.
- Lints : `[workspace.lints.rust] unsafe_code = "forbid"` ;
  `[workspace.lints.clippy] all = { level = "warn", priority = -1 }` ; `[lints] workspace = true`
  dans chaque crate.
- `[profile.release]` : `lto = true`, `codegen-units = 1`, `strip = true`.
- Licence : `license = "MIT OR Apache-2.0"` + fichiers `LICENSE-MIT` et `LICENSE-APACHE`.
- Renommage `crates/josephine-cli/` → `crates/josephine/` : `package.name = "josephine"`,
  MAJ `[workspace] members`, MAJ dépendance `josephine-core = { path = "../josephine-core" }`.
- Métadonnées crate binaire : `keywords`, `categories`, `homepage`, `documentation`,
  `readme`, `description`.
- Squelette : `main.rs` minimal (`#[tokio::main] async fn main() -> ExitCode`) déléguant à un
  `run() -> ExitCode` ; `cli.rs` portant `Cli`/`Commands` + dispatch *thin* (extraits de
  `main.rs`). Mapping erreur → `ExitCode`, ton bienveillant conservé.
- `CHANGELOG.md` (Keep a Changelog + SemVer), entrée `[0.1.0]`.
- `README.md` réécrit en anglais : badges (CI, crates.io, license), description, install,
  usage, development, license. (La doc produit FR existante peut être conservée sous
  `docs/` si utile.)
- `tests/cli.rs` (`assert_cmd` + `predicates`) : `--version`, `--help`, `status`, et
  messages « bientôt » des commandes stub.

**Vérif** : build + test + fmt + clippy verts ; `cargo install --path crates/josephine` OK ;
l'appli répond comme avant.

### Phase 2 — State : refactor migrations

- Créer `crates/josephine-core/migrations/V001__init.sql` à partir du schéma inline de
  `storage.rs::migrate()` (tables `metrics`, `events`, `notifications`, `checks_log` +
  `idx_metrics_check_time`).
- Adopter le pattern template : `const MIGRATIONS: &[&str] = &[include_str!("../migrations/V001__init.sql")]`,
  table `schema_version`, `apply_migrations()` idempotent, test sur `:memory:`.
- Compat ascendante : les `CREATE TABLE IF NOT EXISTS` restent sûrs sur bases existantes.

**Vérif** : test migrations `:memory:` vert ; ouverture d'une base v0.1 existante sans perte.

### Phase 3 — Gouvernance & supply-chain

- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), `SECURITY.md`
  (divulgation coordonnée, contact `k@levilainpetit.dev`), `CONVENTIONS.md`, `CLAUDE.md`
  (guide IA adapté aux axes Joséphine).
- `.github/ISSUE_TEMPLATE/{bug.yml,feature.yml}`, `.github/PULL_REQUEST_TEMPLATE.md`,
  `.github/dependabot.yml`, `.github/CODEOWNERS`.
- `deny.toml` (cargo-deny : licences autorisées `MIT`/`Apache-2.0`/etc., advisories, bans,
  sources).

### Phase 4 — CI

- `.github/workflows/ci.yml` (push + PR sur `main`) :
  - `lint` : `cargo fmt --check`, `cargo clippy -D warnings`.
  - `test` : matrice Ubuntu 22.04/24.04 + Fedora 40/41 (containers).
  - `coverage` : `cargo-tarpaulin`, seuil 80 % (`tarpaulin.toml` avec exclusions), upload
    Codecov.
  - `security` : `cargo-audit` + `cargo-deny`.
  - `bench-smoke` : compile/exécute rapidement les benches.
- `benches/` : un bench criterion smoke (ex. `RulesEngine`).

### Phase 5 — Packaging & release (Linux-pragmatique)

- `Cargo.toml` : `[package.metadata.deb]` + `[package.metadata.generate-rpm]` sur le crate
  binaire.
- `packaging/systemd/josephine.service` : **user unit** (`systemctl --user`), pas de root.
- `packaging/aur/PKGBUILD`, `packaging/homebrew/josephine.rb`.
- `.github/workflows/release.yml` (tag `v*.*.*`) : build Linux x86_64 (+ ARM64 cross si
  faisable), checksums SHA256, archives, publication crates.io, GitHub Release avec
  changelog, deb + rpm + AUR + Homebrew.

### Phase 6 — Site Zola + GitHub Pages

- `site/` (Zola) : `config.toml`, `content/` (hero, features, install par gestionnaire,
  docs rendus depuis `docs/*.md`), `templates/` (thème partagé), `sass/` (brand color
  `#E0A458` injectée).
- `.github/workflows/pages.yml` : `zola build` + `actions/deploy-pages` (project page
  `<user>.github.io/josephine`). API docs → docs.rs (non dupliquées).

### Phase 7 — Vérification finale

- Local : `cargo fmt --check`, `cargo clippy -D warnings`, `cargo build --release`,
  `cargo test` verts ; coverage ≥ 80 % ; `zola build` OK.
- À valider après push/tag GitHub (non vérifiable en local, signalé honnêtement) : matrice
  CI multi-OS, `release.yml` (artefacts/publish), déploiement Pages, upload Codecov.

---

## 5. Hors périmètre (YAGNI)

- Windows/macOS : matrice CI, Scoop, winget, Homebrew-macOS (sauf bascule vers cross-platform
  complet).
- Image Docker (ghcr) + `action.yml` réutilisable (add-on optionnel de la spec template).
- TUI (`ratatui`), helper privilégié — axes non retenus pour Joséphine.
- Implémentation des commandes stub (`clean`, `fix`, `report`) : restent des stubs documentés
  (roadmap v0.2+), hors de cet alignement.
- Documentation bilingue complète du site.

---

## 6. Risques & points d'attention

- **Renommage de crate** : MAJ cohérente de `members`, des `path`, des imports et des chemins
  de packaging. Vérifié par un build complet.
- **Reformatage edition 2024** : un diff de reformatage large mais mécanique ; commit séparé
  pour rester lisible.
- **Coverage 80 %** : les checks système (`/proc`, `systemctl`, `/sys/class/thermal`) sont
  difficiles à tester ; exclusions dans `tarpaulin.toml` et focalisation des tests sur
  `config`, `rules`, `messages`, `storage`/migrations, parsing CLI.
- **Workflows GitHub** : authoring correct mais validation réelle seulement après push/tag.

---

## 7. Ordre d'exécution

Phase par phase, chacune vérifiée avant la suivante. La **Phase 1** livre déjà « le premier
dans ce mood » (standards + parité de code) ; les phases 3→6 montent au niveau open-source
complet. Chaque phase fera l'objet d'un plan d'implémentation détaillé.
