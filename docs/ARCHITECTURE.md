# Joséphine — Architecture

Guide technique pour contribuer au code.

---

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────┐
│  josephine                                          │
│  clap → commands → output (tables, barres, spinner)    │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│  josephine-core                                         │
│                                                         │
│  config ──► checks ──► CheckResult                      │
│                │                                        │
│                ▼                                        │
│         scheduler (démon)                               │
│                │                                        │
│         rules ──► messages ──► notify                   │
│                │                                        │
│                ▼                                        │
│            storage (SQLite)                             │
└─────────────────────────────────────────────────────────┘
```

**Principe :** les checks ne se parlent pas. Le scheduler les exécute, persiste les métriques, passe les seuils au moteur de règles.

---

## Cycle d'un check (démon)

1. Attente `interval_secs` (par check)
2. `check.run()` → `CheckResult`
3. Insertion métriques dans `metrics`
4. `RulesEngine::evaluate_check()` → transitions éventuelles
5. Si transition : event SQLite + notification desktop
6. Log dans `checks_log`

---

## Trait `Check`

```rust
pub trait Check: Send {
    fn name(&self) -> &str;
    fn run(&mut self) -> Result<CheckResult>;
}
```

`CheckResult` contient :

- `metrics` — valeurs numériques avec seuils optionnels (`threshold_warning`, `threshold_critical`)
- `details` — lignes texte pour `doctor`
- `top_processes` — résumé optionnel

**Sévérité :** dérivée des métriques via `metric_severity()` (`check.rs`).

---

## Ajouter un check

1. **Config** — ajouter une section dans `ChecksConfig` (`config.rs`) + defaults + validation
2. **Impl** — `crates/josephine-core/src/checks/foo.rs`
3. **Enregistrement** — `checks/mod.rs` : `build_checks`, `interval_for_check`
4. **Règles** — `scheduler.rs` : `thresholds_for`
5. **Messages** — `messages.rs` : `alert_message` + `recovery_message`
6. **CLI** — `output/style.rs` : `check_label`, `primary_metric` si besoin
7. **Tests** — unitaires sur parsing / seuils ; mock si possible

Ne pas coupler le check à SQLite ou aux notifications directement.

---

## Moteur de règles

Clé d'état : `(check_name, metric_name)`.

Seules les métriques avec `threshold_warning: Some(_)` sont évaluées.

Pour éviter le spam disque : seuils uniquement sur `usage_percent_worst`, pas sur chaque partition.

---

## Notifications

- Textes centralisés dans `messages.rs`
- Interdit dans l'UX : `ERROR`, `FATAL`, `PANIC`
- Titre libnotify : `✨ Joséphine` (`notify.rs`)
- Tests : `messages::tests` vérifient le vocabulaire

---

## CLI output

| Module | Rôle |
|--------|------|
| `runner.rs` | `run_checks_with_progress` + indicatif |
| `status.rs` | Tableau résumé 3 colonnes |
| `doctor.rs` | Panneau par check |
| `bars.rs` | Barres Unicode, couleurs via comfy-table |
| `style.rs` | Bannière, labels, métriques |

**Règle :** pas de codes ANSI `colored` *dans* les cellules de table — utiliser `Cell::fg()`.

---

## Chemins (`paths.rs`)

| Ressource | Chemin |
|-----------|--------|
| Config | `~/.config/josephine/config.yaml` |
| Data dir | `~/.local/share/josephine/` |
| DB | `josephine.db` |
| PID | `daemon.pid` |
| Logs | `daemon.log` |

---

## Démon

- `daemon start` → spawn `josephine --__daemon__`
- Arrêt : SIGTERM via `kill`
- `run_daemon_foreground()` : tracing vers fichier log, handler Ctrl+C

---

## Évolutions prévues (v0.2+)

Voir [ROADMAP.md](ROADMAP.md). Prochaine brique technique : `daemon install` (unité systemd user) + `report`.
