# Joséphine — Roadmap

**Baseline actuelle :** v0.7.0 — voir [CURRENT_STATE.md](CURRENT_STATE.md)

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
| **Logo ASCII** | Retiré au profit d'un en-tête sobre `✦` (v0.7.0) ; `banner.txt` personnalisé toujours possible |
| **Cloud / comptes** | Non — 100 % local |
| **IA pour la surveillance** | Non — observation déterministe uniquement |
| **API REST / interface web** | Piste lointaine (v1.0+) |

---

## v0.2 — Produit installable ✅

**Objectif :** Joséphine utilisable au quotidien sans `cargo run`. **Livré.**

| # | Feature | État |
|---|---------|------|
| 1 | `josephine report` — export texte daté | ✅ |
| 2 | Packaging `.deb`/`.rpm` + install systemd user | ✅ |
| 3 | `josephine notify test` — valider libnotify | ✅ |
| 4 | `josephine update` — self-update (bonus) | ✅ |

**Spec :** [superpowers/specs/2026-06-29-josephine-v02-design.md](superpowers/specs/2026-06-29-josephine-v02-design.md)  
**Plan :** [superpowers/plans/2026-06-29-josephine-v02-plan.md](superpowers/plans/2026-06-29-josephine-v02-plan.md)

---

## v0.3 — Machine complète ✅

| # | Feature | État |
|---|---------|------|
| 1 | Check **réseau** (passerelle, latence, DNS) — LAN uniquement, 100 % local | ✅ |
| 2 | Check **batterie** (laptop) | ✅ |
| 3 | `josephine clean` (aperçu + miniatures) | ✅ |

---

## v0.4 — Accompagnement

| # | Feature | État |
|---|---------|------|
| 1 | `josephine fix` — actions guidées (systemd, espace disque) | ✅ |
| 2 | `config edit` — `$EDITOR` + revalidation | ✅ |
| 3 | Sparklines / synthèse enrichie dans `history` | ✅ |

---

## v0.5 — Prévoyance ✅

Anticiper les pannes, pas seulement les constater.

| # | Feature | État |
|---|---------|------|
| 1 | Check **inodes** (saturation malgré de l'espace libre) | ✅ |
| 2 | Check **SMART** (santé disque, opt-in car root requis) | ✅ |
| 3 | Check **noyau/OOM** (incidents récents via `journalctl -k`) | ✅ |

---

## Machine complète, diffusée — v0.5.0 à v0.7.0 ✅

Entre la « Prévoyance » ci-dessus et la baseline actuelle : internationalisation
(anglais par défaut, français via `language: fr` — v0.5.0), packaging Homebrew
+ AUR livré en assets de release (v0.6.0), puis la **refonte du rendu CLI**
« Constellation sobre » — en-tête `✦` discret, statut porté par des glyphes de
forme et couleur (`●`/`▲`/`✕`, dégradés `[ok]/[!]/[x]` hors TTY), colonnes
alignées, ton détoné « chaleur sobre » en anglais et en français, sur
`status`/`doctor`/`history` et les notifications desktop (v0.7.0).

**Programme d'increments CLI** (ordre A → C → B → D, voir
[superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md](superpowers/specs/2026-07-08-josephine-cli-render-tone-design.md)) :

| Increment | Contenu | État |
|---|---|---|
| A — rendu & ton | `status`/`doctor`/`history` + notifications passés au sobre | ✅ livré (v0.7.0) |
| C — solidité & finitions | bascule de `report`/`clean`/`fix`/`update` sur l'habillage sobre, pied de page unifié, nettoyage de copie EN/FR, docs à jour | ✅ livré (`feat/cli-increment-c`) |
| B — sortie machine & canaux | `--json`, complétions shell, `--help` localisé, canal de notification terminal | ⏳ planifié |
| D — nouveaux checks/commandes | hors périmètre de la refonte CLI | ⏳ planifié |

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
| 2026-07-08 | Refonte CLI « Constellation sobre » (ton *chaleur sobre*, increment A) actée |
| 2026-07-09 | v0.7.0 : increment A livré (rendu + notifications) ; increment C (finitions) démarré |
