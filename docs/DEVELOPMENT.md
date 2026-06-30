# Joséphine — Guide de développement

---

## Prérequis

- Rust ≥ 1.75 (stable)
- Linux pour exécuter les checks réels
- `libnotify-bin` pour tester les notifications desktop

---

## Commandes utiles

```bash
# Build
cargo build
cargo build --release

# Tests
cargo test
cargo test -p josephine-core

# Exécution locale
cargo run -p josephine-cli -- status
cargo run -p josephine-cli -- doctor
cargo run -p josephine-cli -- daemon start

# Installation binaire
cargo install --path crates/josephine-cli
```

---

## Structure du workspace

```
josephine/
├── Cargo.toml              # workspace + deps partagées
├── crates/
│   ├── josephine-core/     # lib métier
│   └── josephine-cli/      # binaire josephine
└── docs/                   # specs, roadmap, architecture
```

---

## Conventions

### Code

- Messages utilisateur et notifications : **français**
- Ton bienveillant, jamais alarmiste
- Changements minimaux par PR / commit logique
- Un check = un fichier dans `checks/`
- Textes de notification uniquement dans `messages.rs`

### Config

- Nouveau check → section YAML + validation explicite
- Intervalle minimum : 5 secondes
- Seuils percentage : 0–100, `warning < critical`

### CLI

- Commandes interactives : spinner (`run_checks_with_progress`)
- Respect `NO_COLOR` / non-TTY : pas de couleurs ni spinner
- Stubs : message « bientôt » via `stub_cmd.rs`

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
- [ ] Messages notification (si alertes)
- [ ] Affichage CLI (`status` / `doctor` / `history`)
- [ ] README si commande utilisateur visible
- [ ] CURRENT_STATE.md mis à jour
