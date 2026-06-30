# Joséphine v0.2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre Joséphine installable au quotidien via `report`, systemd user, et tests.

**Architecture:** Extensions CLI + module `daemon/install.rs` dans core, formatteur plain pour report.

**Tech Stack:** Rust, clap, systemd user units, fichiers texte UTF-8.

**Spec:** [2026-06-29-josephine-v02-design.md](../specs/2026-06-29-josephine-v02-design.md)

---

## Task 1: Module report (texte plain)

**Files:**
- Create: `crates/josephine-cli/src/output/report.rs`
- Create: `crates/josephine-cli/src/commands/report_cmd.rs`
- Modify: `crates/josephine-cli/src/main.rs`
- Modify: `crates/josephine-cli/src/commands/mod.rs`
- Modify: `crates/josephine-cli/src/commands/stub_cmd.rs` — retirer Report du stub

- [ ] **Step 1:** Créer `format_report(results, daemon_status, history_summary) -> String` sans couleurs
- [ ] **Step 2:** Ajouter flags clap `-o/--output`, `--stdout`
- [ ] **Step 3:** Nom de fichier par défaut `josephine-report-YYYY-MM-DD-HHMM.txt`
- [ ] **Step 4:** Remplacer stub `report` par implémentation réelle
- [ ] **Step 5:** `cargo run -- report` et vérifier contenu

---

## Task 2: daemon install / uninstall

**Files:**
- Create: `crates/josephine-core/src/daemon_install.rs`
- Modify: `crates/josephine-core/src/lib.rs`
- Modify: `crates/josephine-cli/src/commands/daemon_cmd.rs`

- [ ] **Step 1:** `generate_unit_file(exe_path: &Path) -> String`
- [ ] **Step 2:** `install_user_service()` — écrit `~/.config/systemd/user/josephine.service`, `systemctl --user daemon-reload`, `enable --now`
- [ ] **Step 3:** `uninstall_user_service()` — inverse proprement
- [ ] **Step 4:** Sous-commandes clap `daemon install`, `daemon uninstall`
- [ ] **Step 5:** Messages bienveillants si systemd absent

---

## Task 3: notify test (optionnel)

**Files:**
- Create: `crates/josephine-cli/src/commands/notify_cmd.rs`
- Modify: `crates/josephine-core/src/messages.rs` — message test dédié

- [ ] **Step 1:** Commande `josephine notify test`
- [ ] **Step 2:** Appelle `notify::send_josephine` avec message fixe drôle
- [ ] **Step 3:** Code sortie 1 si libnotify échoue

---

## Task 4: Tests d'intégration

**Files:**
- Create: `crates/josephine-core/tests/config_roundtrip.rs`
- Create: `crates/josephine-core/tests/thermal_fixtures.rs`

- [ ] **Step 1:** Test sérialisation config default → YAML → parse → validate
- [ ] **Step 2:** Test `read_thermal_zones` sur répertoire fixture temporaire
- [ ] **Step 3:** `cargo test` vert

---

## Task 5: Documentation & packaging

**Files:**
- Modify: `README.md`
- Modify: `docs/CURRENT_STATE.md`
- Modify: `docs/ROADMAP.md`
- Create: `packaging/deb/README.md` (instructions build manuel)

- [ ] **Step 1:** Section « Installation production » dans README
- [ ] **Step 2:** Documenter `daemon install` + `report`
- [ ] **Step 3:** Mettre CURRENT_STATE à jour post-livraison

---

## Ordre recommandé

1. Task 1 (report) — valeur utilisateur immédiate  
2. Task 2 (systemd user) — autonomie démon  
3. Task 4 (tests) — confiance  
4. Task 3 (notify test) — confort dev  
5. Task 5 (docs + packaging)
