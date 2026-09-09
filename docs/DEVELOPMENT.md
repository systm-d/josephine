# Joséphine — Guide de développement

---

## Prérequis

- Rust ≥ 1.85 (stable) — `rust-version` du workspace
- Linux pour exécuter les checks réels
- `libnotify-bin` pour tester les notifications desktop

---

## Commandes utiles

```bash
# Build
cargo build
cargo build --release

# Tests
cargo test --workspace
cargo test -p josephine-core

# Barrière qualité (avant chaque PR)
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

# Exécution locale
cargo run -p josephine -- status
cargo run -p josephine -- doctor
cargo run -p josephine -- daemon start

# Installation binaire
cargo install --path crates/josephine
```

---

## Structure du workspace

```
josephine/
├── Cargo.toml              # workspace + deps partagées
├── crates/
│   ├── josephine-core/     # lib métier
│   └── josephine/          # binaire josephine
└── docs/                   # specs, roadmap, architecture
```

---

## Conventions

### Code

- Toute chaîne visible par l'utilisateur existe en **anglais et en français** —
  `i18n::t(en, fr)`, ou `match i18n::lang()` quand il y a interpolation.
  L'anglais est la langue par défaut, le français s'active avec `language: fr`
- Ton bienveillant, jamais alarmiste ; jamais `ERROR` / `FATAL` / `PANIC`
- Identifiants de code en anglais
- Changements minimaux par PR / commit logique
- Un check = un fichier dans `checks/`
- Textes de notification uniquement dans `messages.rs`
- Ce qu'un check surveille, pourquoi et comment agir : une entrée `Advice` dans
  `remedy.rs` — partagée par `josephine explain` et la section finale de
  `doctor`. Un test échoue si elle manque
- Formulations de « voix » (salutations, sign-offs) : `voice.rs` — de la
  variété uniquement, jamais les faits d'une alerte

### Config

- Nouveau check → section YAML + validation explicite
- Intervalle minimum : 5 secondes
- Seuils percentage : 0–100, `warning < critical`

### CLI

- Commandes interactives : spinner (`run_checks_with_progress`)
- Respect `NO_COLOR` / non-TTY : pas de couleurs ni spinner
- Gravité portée par la forme *et* la couleur (`●` / `▲` / `✕`), lisible dans un pipe

### Tests

- Obligatoires pour : règles, config, messages, parsing (systemd, etc.)
- Pas de dépendance réseau en CI
- Pas de mock lourd : préférer fonctions pures testables

---

## Déboguer le démon

```bash
josephine daemon start
josephine daemon status
josephine daemon logs
tail -f ~/.local/share/josephine/daemon.log

# Arrêt
josephine daemon stop
```

Forcer une alerte : baisser temporairement un seuil dans `~/.config/josephine/config.yaml`, puis `daemon restart`.

---

## Déboguer les notifications

1. `notifications.desktop: true`
2. Démon actif
3. `libnotify` installé + serveur de notifications actif (GNOME, KDE…)
4. Vérifier les logs si `notify-rust` échoue

---

## Documents à lire avant une feature

1. [CURRENT_STATE.md](CURRENT_STATE.md) — baseline
2. [ROADMAP.md](ROADMAP.md) — priorité produit
3. Spec de version cible (`docs/superpowers/specs/`)
4. [ARCHITECTURE.md](ARCHITECTURE.md) — où brancher le code

---

## Checklist nouvelle fonctionnalité

- [ ] Spec ou section roadmap mise à jour
- [ ] Config + validation si applicable
- [ ] Tests unitaires
- [ ] Messages notification (si alertes), en anglais **et** en français
- [ ] Entrée `Advice` dans `remedy.rs` si c'est un nouveau check
- [ ] Affichage CLI (`status` / `doctor` / `history`)
- [ ] README.md **et** README.fr.md si commande utilisateur visible
- [ ] CHANGELOG.md sous `[Unreleased]`
- [ ] CURRENT_STATE.md mis à jour
