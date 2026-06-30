# Joséphine v0.1 — Design Document

> **Statut : LIVRÉ** (v0.1.0)  
> **Référence à jour :** [docs/CURRENT_STATE.md](../../CURRENT_STATE.md)  
> Ce document reste l'historique de conception initiale ; les écarts sont documentés en section 21.

**Date :** 2026-06-29  
**Langage :** Rust  
**Cible :** Debian 13+ (Linux)  
**Licence :** MIT  

---

## 1. Objectif v0.1

Livrer une boucle de surveillance complète mais limitée :

- CLI : `status`, `doctor`, `daemon`, `config`, `history`
- Démon en arrière-plan avec checks CPU, RAM, disque
- Moteur de règles (NORMAL → WARNING → CRITICAL → RECOVERED)
- Notifications desktop (libnotify)
- Historique SQLite local
- Stubs pour `clean`, `fix`, `report`, `config edit`

---

## 2. Architecture

```
josephine-cli (binaire unique)
    │
    ├── commandes CLI
    │
    └── josephine-core (lib)
            ├── config/       ~/.config/josephine/config.yaml
            ├── checks/       cpu, memory, disk (indépendants)
            ├── scheduler/    boucle tokio, intervalles par check
            ├── rules/        états et transitions
            ├── storage/      SQLite
            └── notify/       notify-rust
```

**Workspace Cargo :**

| Crate | Rôle |
|-------|------|
| `josephine-core` | Logique métier, checks, règles, stockage |
| `josephine-cli` | Point d'entrée, parsing clap, affichage |

Le démon est le même binaire : `josephine daemon start` lance `josephine __daemon__` en arrière-plan.

---

## 3. Stack technique

| Rôle | Crate |
|------|-------|
| CLI | `clap` 4 |
| Async | `tokio` (full) |
| Métriques | `sysinfo` |
| Config | `serde`, `serde_yaml` |
| SQLite | `rusqlite` (bundled) |
| Notifications | `notify-rust` |
| Affichage | `comfy-table`, `colored` |
| Logs | `tracing`, `tracing-subscriber` |
| Erreurs | `anyhow`, `thiserror` |
| Temps | `chrono` |
| Dirs | `dirs` |

---

## 4. Structure des fichiers

```
josephine/
├── Cargo.toml
├── README.md
├── crates/
│   ├── josephine-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── paths.rs
│   │       ├── check.rs          # trait Check
│   │       ├── checks/
│   │       │   ├── mod.rs
│   │       │   ├── cpu.rs
│   │       │   ├── memory.rs
│   │       │   └── disk.rs
│   │       ├── rules.rs
│   │       ├── scheduler.rs
│   │       ├── storage.rs
│   │       ├── notify.rs
│   │       └── daemon.rs
│   └── josephine-cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── output.rs
│           └── commands/
│               ├── mod.rs
│               ├── status.rs
│               ├── doctor.rs
│               ├── daemon.rs
│               ├── config.rs
│               ├── history.rs
│               └── stub.rs
```

---

## 5. Chemins système

| Ressource | Chemin |
|-----------|--------|
| Config | `~/.config/josephine/config.yaml` |
| Base SQLite | `~/.local/share/josephine/josephine.db` |
| PID démon | `~/.local/share/josephine/daemon.pid` |
| Logs démon | `~/.local/share/josephine/daemon.log` |

Création automatique des répertoires au premier lancement.

---

## 6. Configuration YAML

```yaml
checks:
  cpu:
    enabled: true
    interval_secs: 30
    warning: 85
    critical: 95
  memory:
    enabled: true
    interval_secs: 60
    warning: 85
    critical: 95
  disk:
    enabled: true
    interval_secs: 120
    warning: 85
    critical: 95

notifications:
  desktop: true
  terminal: false

history:
  enabled: true
  retention_days: 90
```

Validation : seuils entre 0–100, intervalles ≥ 5 s.

---

## 7. Schéma SQLite

### Table `metrics`

| Colonne | Type | Description |
|---------|------|-------------|
| id | INTEGER PK | |
| check_name | TEXT | cpu, memory, disk |
| metric_name | TEXT | usage_percent, swap_percent, … |
| value | REAL | |
| recorded_at | TEXT ISO8601 | |

Index : `(check_name, recorded_at)`.

### Table `events`

| Colonne | Type | Description |
|---------|------|-------------|
| id | INTEGER PK | |
| check_name | TEXT | |
| metric_name | TEXT | |
| from_state | TEXT | |
| to_state | TEXT | NORMAL/WARNING/CRITICAL/RECOVERED |
| value | REAL | |
| message | TEXT | message bienveillant |
| created_at | TEXT ISO8601 | |

### Table `notifications`

| Colonne | Type | Description |
|---------|------|-------------|
| id | INTEGER PK | |
| event_id | INTEGER FK | |
| channel | TEXT | desktop |
| sent_at | TEXT ISO8601 | |

### Table `checks_log`

| Colonne | Type | Description |
|---------|------|-------------|
| id | INTEGER PK | |
| check_name | TEXT | |
| status | TEXT | ok, error |
| duration_ms | INTEGER | |
| error_message | TEXT nullable | |
| ran_at | TEXT ISO8601 | |

Purge automatique : métriques et événements plus vieux que `retention_days`.

---

## 8. Trait `Check`

```rust
pub struct CheckResult {
    pub check_name: String,
    pub metrics: Vec<Metric>,
    pub details: Vec<String>,      // lignes pour doctor
    pub top_processes: Vec<String>, // optionnel
}

pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,              // "%", "°C", "GiB"
}

pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self) -> anyhow::Result<CheckResult>;
}
```

Chaque check est indépendant, sans communication inter-checks.

---

## 9. Checks v0.1

### CPU

- Utilisation globale (%)
- Load average 1/5/15 min
- Top 3 processus par CPU
- Seuils : warning/critical sur `usage_percent`

### Memory

- RAM utilisée (%)
- Swap utilisé (%)
- Top 3 processus par RAM
- Seuils : warning/critical sur `usage_percent` (RAM)

### Disk

- Pour chaque partition montée (hors tmpfs, squashfs, devtmpfs) :
  - usage % espace
  - usage % inodes (si disponible)
- Seuils : warning/critical sur la partition la plus remplie

---

## 10. Moteur de règles

État en mémoire : `HashMap<(check_name, metric_name), AlertState>`.

```rust
enum AlertState { Normal, Warning, Critical }
```

Transitions :

| De → À | Action |
|--------|--------|
| Normal → Warning | Notification + event |
| Normal → Critical | Notification + event |
| Warning → Critical | Notification + event |
| * → Normal (depuis Warning/Critical) | Notification RECOVERED + event |
| Warning → Warning | Rien (pas de spam) |
| Critical → Critical | Rien |

Comparaison : `value >= critical` → Critical ; `value >= warning` → Warning ; sinon Normal.

---

## 11. Notifications

Crate `notify-rust`. Titre : `✨ Joséphine`. Corps : message bienveillant en français.

Exemples :

- « Votre CPU reste au-dessus de 85 %. Processus principal : firefox (PID 1234). »
- « Votre disque approche de sa limite (92 % sur /). Je peux vous aider à identifier ce qui prend de la place. »
- « Bonne nouvelle : votre utilisation CPU est revenue à la normale. »

Désactivables via `notifications.desktop: false`.

---

## 12. Scheduler (démon)

Boucle tokio avec une tâche par check activé :

1. Attendre `interval_secs`
2. Exécuter `check.run()`
3. Persister métriques en SQLite
4. Passer métriques seuillables au moteur de règles
5. Émettre notifications si transition
6. Logger durée dans `checks_log`

Intervalle minimum global entre wakeups : respect des intervales par check (pas de polling agressif).

---

## 13. Gestion du démon

| Commande | Comportement |
|----------|--------------|
| `daemon start` | Vérifie PID file ; si absent, spawn `josephine __daemon__` via `std::process::Command` avec stdout/stderr redirigés vers log |
| `daemon stop` | Envoie SIGTERM au PID, supprime PID file |
| `daemon status` | Affiche running/stopped + uptime si running |
| `daemon restart` | stop + start |
| `daemon logs` | Affiche dernières 50 lignes du fichier log |

Signal SIGTERM : arrêt propre (flush SQLite, suppression PID file).

---

## 14. Commandes CLI

### `status`

Table compacte : CPU, RAM, disque (/ ou pire partition), état global (✓ / ⚠ / ✗).

### `doctor`

Sections par check avec détails, top processus, recommandations douces.

### `history`

Dernières 24 h : max CPU/RAM/temp (temp stub N/A v0.1), événements récents.

### Stubs

`clean`, `fix`, `report`, `config edit` → message :

```
Cette fonctionnalité arrive bientôt avec Joséphine.
```

---

## 15. Ton UX

- Pas de `ERROR`, `FATAL`, `PANIC` dans la sortie utilisateur
- Niveaux : `info`, `attention`, `critique`
- Messages en français
- Codes de sortie CLI : 0 succès, 1 erreur générique, 2 config invalide

---

## 16. Consommation cible

- RAM démon < 30 Mo
- CPU moyen < 0,5 %
- Pas de requêtes réseau

---

## 17. Tests v0.1

- Tests unitaires : moteur de règles (transitions, anti-spam)
- Tests unitaires : parsing config
- Tests unitaires : comparaison seuils
- Pas de tests d'intégration système en CI (dépendent de /proc)

---

## 18. Hors périmètre v0.1

- Batterie, réseau
- `clean`, `fix`, `report`
- Plugins, API REST, interface web
- `config edit` interactif

*(Température et systemd ont été livrés après v0.1.)*

---

## 19. Hors périmètre produit (décision)

- **Docker** — hors mission « gardien machine »
- **TUI / `watch`** — contraire au principe « invisible par défaut »

---

## 20. Évolutions v0.2+

Spec détaillée : `docs/superpowers/specs/2026-06-29-josephine-v02-design.md`  
Plan d’implémentation : `docs/superpowers/plans/2026-06-29-josephine-v02-plan.md`  
Vue d’ensemble : `docs/ROADMAP.md`

---

## 21. Écarts par rapport à la livraison v0.1.0

| Spec initiale | Livré |
|---------------|-------|
| Checks cpu, memory, disk | ✅ + temperature, systemd |
| Affichage basique | ✅ indicatif, comfy-table, barres |
| Messages notifications neutres | ✅ module `messages.rs`, ton *ange gardien* |
| Commande `watch` | ❌ retirée (hors scope TUI) |
| Logo ASCII | ❌ reporté |
| `notifications.terminal` | Config seulement, non implémenté |
| Structure `output.rs` unique | ✅ module `output/` découpé |
| Flag démon `__daemon__` | ✅ `--__daemon__` (clap long) |
| 5 tests | ✅ 10 tests unitaires |
