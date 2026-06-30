# Joséphine v0.2 — Design Document

**Date :** 2026-06-29  
**Statut :** À implémenter  
**Prérequis :** v0.1.0 livré — voir [CURRENT_STATE.md](../../CURRENT_STATE.md)

---

## 1. Objectif v0.2

Rendre Joséphine **installable et autonome** au quotidien, sans ajouter de nouveaux checks.

**Livrables :**

1. `josephine report`
2. `josephine daemon install` / `uninstall`
3. Fichier `.deb` (optionnel mais documenté)
4. Tests d'intégration ciblés
5. (Optionnel) `josephine notify test`

**Hors v0.2 :** réseau, batterie, clean, fix, logo.

---

## 2. `josephine report`

### Comportement

```bash
josephine report              # stdout
josephine report -o ~/rapport.txt
josephine report --stdout     # explicite
```

### Contenu

- En-tête : date, hostname, version Joséphine
- Résultat de tous les checks (comme `doctor`, format texte)
- État du démon
- Synthèse history 24 h (max métriques, derniers événements)
- Recommandations douces (ton *ange gardien*)

### Format

- Fichier par défaut : `josephine-report-YYYY-MM-DD-HHMM.txt` dans le cwd
- Encodage UTF-8
- Pas de couleur (texte plain)

### Implémentation

- Nouveau module `josephine-cli/src/commands/report_cmd.rs`
- Réutiliser `run_checks_with_progress` + formatteur plain dans `output/report.rs`
- Pas de SQLite supplémentaire

---

## 3. `josephine daemon install`

### Comportement

```bash
josephine daemon install    # unité systemd user
josephine daemon uninstall  # retire l'unité
```

### Unité systemd user

Fichier : `~/.config/systemd/user/josephine.service`

```ini
[Unit]
Description=Joséphine — ange gardien de votre ordinateur
After=default.target

[Service]
ExecStart=%h/.cargo/bin/josephine --__daemon__
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

**Notes :**

- Résoudre le chemin réel via `std::env::current_exe()` au moment de `install`
- Exécuter `systemctl --user daemon-reload` + `enable --now`
- Message bienveillant si systemd indisponible
- `uninstall` : `disable --now`, supprimer le fichier unit

### Alternative v0.2

Si systemd user trop fragile : documenter uniquement `daemon start` + entrée crontab `@reboot` — **préférer systemd user**.

---

## 4. Packaging

### cargo install

Déjà supporté :

```bash
cargo install --path crates/josephine-cli
```

### .deb (stretch goal v0.2)

- Script `packaging/deb/build.sh` ou `cargo deb` si crate configuré
- Dépendances : `libnotify-bin`
- Binaire dans `/usr/bin/josephine`
- Fichier conffiles : non (config dans `$HOME`)

Documenter dans README ; implémentation minimale acceptable.

---

## 5. Tests d'intégration

Cible : `josephine-core/tests/`

| Test | Approche |
|------|----------|
| Config round-trip | YAML → validate → serialize |
| Rules + messages | transitions + contenu message |
| Temperature parse | fixtures sysfs dans `tests/fixtures/thermal/` |
| Systemd parse | mock sortie `systemctl` via fonction injectée ou tests unitaires existants |

Pas de CI obligatoire dans v0.2 si hors scope infra — tests locaux suffisent.

---

## 6. `josephine notify test` (optionnel)

```bash
josephine notify test
```

Envoie une notification desktop de test avec un message *ange gardien*. Utile pour valider libnotify sans baisser les seuils.

---

## 7. Documentation à mettre à jour

- [CURRENT_STATE.md](../../CURRENT_STATE.md)
- [README.md](../../../README.md)
- Passer v0.2 spec → « Livré » quand terminé

---

## 8. Critères de done v0.2

- [ ] `report` génère un fichier lisible
- [ ] `daemon install` active le service user
- [ ] `daemon uninstall` nettoie proprement
- [ ] README installation production
- [ ] ≥ 2 tests d'intégration additionnels
- [ ] CURRENT_STATE + ROADMAP à jour
