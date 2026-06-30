# Joséphine — guide agent / contributeur

Projet : gardien système Linux local, Rust, workspace `josephine-core` + `josephine-cli`.

## Lire en premier

1. [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md) — ce qui existe
2. [docs/ROADMAP.md](docs/ROADMAP.md) — priorités
3. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — comment le code est organisé

## Règles produit

- Français, ton bienveillant (*Joséphine, ange gardien*), jamais `ERROR`/`FATAL`/`PANIC` en UX
- 100 % local, pas de cloud
- **Hors scope :** Docker, TUI/`watch`, logo ASCII (reporté)

## Où modifier quoi

| Besoin | Fichier |
|--------|---------|
| Nouveau check | `crates/josephine-core/src/checks/` + `config.rs` + `messages.rs` |
| Textes notifications | `crates/josephine-core/src/messages.rs` |
| Affichage CLI | `crates/josephine-cli/src/output/` |
| Commande CLI | `crates/josephine-cli/src/commands/` |

## Commandes dev

```bash
cargo test
cargo run -p josephine-cli -- status
cargo run -p josephine-cli -- doctor
```

## Prochaine version

Spec v0.2 : [docs/superpowers/specs/2026-06-29-josephine-v02-design.md](docs/superpowers/specs/2026-06-29-josephine-v02-design.md)  
Plan : [docs/superpowers/plans/2026-06-29-josephine-v02-plan.md](docs/superpowers/plans/2026-06-29-josephine-v02-plan.md)
