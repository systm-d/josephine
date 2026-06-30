# Joséphine — Roadmap

**Baseline actuelle :** v0.1.0 — voir [CURRENT_STATE.md](CURRENT_STATE.md)

---

## v0.1 — Livré ✅

Surveillance locale silencieuse de la **machine** :

- Checks : CPU, RAM, disque, température, systemd
- CLI : `status`, `doctor`, `history`, `daemon`, `config`
- Démon, règles anti-spam, notifications desktop (ton *ange gardien*)
- Historique SQLite, affichage CLI (indicatif, comfy-table)

---

## Hors périmètre (décisions actées)

| Sujet | Décision |
|-------|----------|
| **Docker** | Non — outils dédiés, hors mission « gardien machine » |
| **TUI / `watch`** | Non — contraire au principe « invisible par défaut » |
| **Logo ASCII** | Reporté |
| **Cloud / comptes** | Non — 100 % local |
| **IA pour la surveillance** | Non — observation déterministe uniquement |
| **API REST / interface web** | Piste lointaine (v1.0+) |

---

## v0.2 — Produit installable

**Objectif :** Joséphine utilisable au quotidien sans `cargo run`.

| # | Feature | Priorité |
|---|---------|----------|
| 1 | `josephine report` — export texte daté | Haute |
| 2 | `josephine daemon install` — unité systemd user | Haute |
| 3 | Packaging `.deb` + doc installation | Moyenne |
| 4 | Tests d'intégration checks (mocks sysfs / systemctl) | Moyenne |
| 5 | `josephine notify test` — valider libnotify | Basse |

**Spec :** [superpowers/specs/2026-06-29-josephine-v02-design.md](superpowers/specs/2026-06-29-josephine-v02-design.md)  
**Plan :** [superpowers/plans/2026-06-29-josephine-v02-plan.md](superpowers/plans/2026-06-29-josephine-v02-plan.md)

---

## v0.3 — Machine complète

| # | Feature |
|---|---------|
| 1 | Check **réseau** (DNS, passerelle, latence) |
| 2 | Check **batterie** (laptop) |
| 3 | `josephine clean --dry-run` (caches système, `/tmp`, journaux) |

---

## v0.4 — Accompagnement

| # | Feature |
|---|---------|
| 1 | `josephine fix` — actions guidées (systemd, espace disque) |
| 2 | `config edit` — `$EDITOR` + revalidation |
| 3 | Sparklines / synthèse enrichie dans `history` |

---

## v1.0+ — Écosystème (non planifié en détail)

- Plugins internes (postgres, nginx…)
- API REST locale
- Interface web localhost
- `josephine explain` (couche explication, pas surveillance)

---

## Critère d'ajout

Chaque feature doit répondre **oui** à :

> Est-ce que cela aide Joséphine à mieux **protéger la machine** ?

Si non → autre outil.

---

## Historique des décisions

| Date | Décision |
|------|----------|
| 2026-06-29 | Rust, workspace core + cli |
| 2026-06-29 | Option B v0.1 : boucle démon + 3 checks initiaux |
| 2026-06-29 | + température, + systemd |
| 2026-06-29 | Docker et TUI sortis du scope |
| 2026-06-29 | Messages notifications : ton série *ange gardien* |
| 2026-06-29 | Logo ASCII reporté |
