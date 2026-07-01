# Joséphine — État actuel du code

**Version :** 0.1.0  
**Dernière mise à jour :** 2026-06-29  
**Langage :** Rust (workspace Cargo)  
**Cible :** Linux (Debian 13+ recommandé)

Ce document est la **source de vérité** pour l'état du dépôt. En cas de divergence avec la spec v0.1 initiale, **ce fichier prévaut**.

---

## Livré

### Checks (5)

| Check | Métriques principales | Source |
|-------|----------------------|--------|
| `cpu` | `usage_percent`, load avg, top 3 processus | `sysinfo` |
| `memory` | `usage_percent`, `swap_percent`, top 3 processus | `sysinfo` |
| `disk` | partitions montées, `usage_percent_worst` | `sysinfo` |
| `temperature` | `temp_max_celsius` | `/sys/class/thermal`, NVMe hwmon |
| `systemd` | `failed_units`, `max_restarts` | `systemctl` |

Chaque check implémente le trait `Check` (`josephine-core/src/check.rs`), est indépendant, configurable via YAML.

### CLI

| Commande | Statut |
|----------|--------|
| `status` (défaut) | ✅ |
| `doctor` | ✅ |
| `history` | ✅ |
| `daemon start/stop/restart/status/logs` | ✅ |
| `config show/validate` | ✅ |
| `update` (`--check`, `--yes`) | ✅ |
| `clean`, `fix`, `report` | stub |
| `config edit` | stub |

**Supprimé du scope :** `watch` (TUI), check Docker.

### Démon

- Binaire unique : `josephine --__daemon__` (flag interne)
- PID : `~/.local/share/josephine/daemon.pid`
- Logs : `~/.local/share/josephine/daemon.log`
- Scheduler tokio : une tâche async par check activé

### Règles & notifications

- États : NORMAL → WARNING → CRITICAL → RECOVERED
- Anti-spam : pas de notification si l'état ne change pas
- Messages : module `messages.rs`, ton « Joséphine ange gardien » (humour bienveillant, jamais alarmiste)
- Canal : desktop via `notify-rust` / libnotify
- `notifications.terminal` : présent en config, **non implémenté**

### Stockage

- SQLite : `~/.local/share/josephine/josephine.db`
- Tables : `metrics`, `events`, `notifications`, `checks_log`
- Purge horaire selon `history.retention_days`

### Affichage CLI

- Module `josephine/src/output/` : `bars`, `status`, `doctor`, `runner`, `style`
- `indicatif` : progression pendant les checks (`status`, `doctor`)
- `comfy-table` + couleurs via API table (pas d'ANSI dans les cellules)
- Mode plain si sortie non-TTY
- **Pas de logo ASCII** (reporté)

---

## Configuration

Fichier : `~/.config/josephine/config.yaml`

Structures notables :

- `CheckThresholds` — cpu, memory, disk (%, intervalles)
- `TemperatureThresholds` — seuils en °C (20–150)
- `SystemdCheckConfig` — seuils `failed_*` et `restarts_*` (comptes entiers ≥ 1)

Validation dans `config.rs::validate()`.

---

## Tests

10 tests unitaires dans `josephine-core` :

- `config` (2)
- `rules` (3)
- `messages` (3)
- `checks/systemd` (2)

Pas de tests d'intégration système en CI (dépendance `/proc`, `systemctl`).

Commande : `cargo test`

---

## Stack (workspace)

| Crate | Usage |
|-------|--------|
| clap 4 | CLI |
| tokio | Démon / scheduler |
| sysinfo | CPU, RAM, disque |
| rusqlite | Historique |
| notify-rust | Notifications |
| serde_yaml | Config |
| indicatif | Barre de progression CLI |
| comfy-table | Tableaux |
| colored | Bannières, résumés |
| tracing | Logs démon |

---

## Décisions produit actées

| Décision | Détail |
|----------|--------|
| Langage | Rust (pas Python du PVD original) |
| Docker | Hors périmètre |
| TUI / `watch` | Hors périmètre |
| Logo ASCII | Reporté |
| Notifications | Démon uniquement, pas le CLI interactif |
| Ton | Français, bienveillant, référence série *ange gardien* |

---

## Fichiers clés

```
crates/josephine-core/src/
  check.rs           trait Check
  checks/            implémentations
  config.rs          YAML + validation
  rules.rs           moteur d'états
  messages.rs        textes notifications
  scheduler.rs       boucle démon + run_all_checks
  storage.rs         SQLite
  daemon.rs          start/stop/status
  notify.rs          libnotify

crates/josephine/src/
  main.rs            clap
  output/            rendu terminal
  commands/          sous-commandes
```

---

## Prochaine étape documentée

Voir [ROADMAP.md](ROADMAP.md) et [superpowers/specs/2026-06-29-josephine-v02-design.md](superpowers/specs/2026-06-29-josephine-v02-design.md).
