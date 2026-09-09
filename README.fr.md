<p align="center">
  <img src="resources/social-preview-fr.png" alt="Joséphine — l'ange gardien discret de votre ordinateur" width="720">
</p>

<h1 align="center">Joséphine</h1>

<p align="center"><em>L'ange gardien discret de votre ordinateur.</em></p>

<p align="center">
  <a href="https://github.com/systm-d/josephine/actions/workflows/ci.yml"><img src="https://github.com/systm-d/josephine/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/systm-d/josephine/releases/latest"><img src="https://img.shields.io/github/v/release/systm-d/josephine?color=e0a458&label=release" alt="Dernière version"></a>
  <img src="https://img.shields.io/badge/platform-Linux-333" alt="Linux uniquement">
  <a href="#licence"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="Licence : MIT OR Apache-2.0"></a>
</p>

Joséphine observe votre machine en silence et ne prend la parole que lorsque
c'est utile. Elle garde un œil sur le processeur, la mémoire, le disque, la
température, les services systemd et les mises à jour en attente, repère les
ennuis tôt, et envoie des notifications desktop chaleureuses, en français
courant — jamais intrusives, toujours locales. **Aucune donnée ne quitte votre
ordinateur.**

> Joséphine parle **anglais par défaut** ; mettez `language: fr` dans
> `~/.config/josephine/config.yaml` pour sa voix française. La même voix
> chaleureuse et protectrice dans les deux langues.

## Fonctionnalités

- **Seize contrôles intégrés** — processeur, mémoire, disque, température,
  services systemd, mises à jour de paquets (apt / dnf / pacman), réseau local
  (latence vers la passerelle), batterie, inodes, santé SMART et usure
  SSD/NVMe (opt-in), incidents noyau (OOM / oops), systèmes de fichiers
  remontés en lecture seule, synchronisation NTP de l'horloge, connexions
  échouées récentes, redémarrage en attente après une mise à jour du noyau ou
  d'une bibliothèque, et pression système (PSI : blocages mémoire/CPU/IO,
  avant que l'OOM killer n'intervienne).
- **Notifications chaleureuses** — des alertes desktop en langue claire, qui ne
  montent en intensité que si cela aide ; jamais `ERROR` / `FATAL` / `PANIC`.
  Elle varie ses formulations pour ne jamais sonner préenregistré — pendant que
  les faits (le chiffre, la commande) restent exacts.
- **Démon en arrière-plan** — un service systemd *utilisateur* léger qui veille
  en continu et tient un **historique glissant de 24 h** (SQLite local).
- **`status` d'un coup d'œil** — résumé coloré, avec une **bannière
  personnalisable**.
- **`doctor` détaillé** — il ouvre sur un verdict clair (tout va bien, une
  remarque ou deux, ou quelque chose qui a besoin de vous maintenant), puis
  détaille contrôle par contrôle ; `--verbose` ajoute les seuils, les 10
  processus les plus gourmands et l'intervalle de collecte de chaque contrôle.
- **Prévoyance** — `doctor` se termine sur un mot d'avance quand une tendance
  lente mène quelque part dans le mois (un disque qui se remplit, des inodes
  qui s'épuisent, un SSD usé) : *« Disque : plein dans ~6 jours »*. Une
  projection simple et déterministe de l'historique local — jamais une
  supposition, jamais une notification, seulement quand la tendance est réelle
  et proche.
- **`explain`** — ce que surveille chaque contrôle, pourquoi cela compte, et
  comment agir.
- **Mise à jour autonome** — `josephine update` interroge les GitHub Releases et
  installe le paquet correspondant à votre installation (`.deb` / `.rpm`) ; ne
  touche au réseau que lorsque vous le demandez.
- **Bilingue** — anglais par défaut, français avec `language: fr` dans la
  configuration ; chaleureuse et jamais alarmiste dans les deux.
- **100 % local** — pas de cloud, pas de télémétrie, native Linux (systemd,
  `/sys`, libnotify).

## Installation

Chaque tag `v*.*.*` déclenche le workflow [Release](.github/workflows/release.yml),
qui publie des artefacts pour les cibles Linux courantes. Joséphine est native
Linux (systemd, `/sys`, `/proc`, libnotify) — il n'y a **aucune build macOS ou
Windows**.

| Plateforme      | Artefact                          |
| --------------- | --------------------------------- |
| Linux (générique) | `josephine-linux-x86_64.tar.gz` |
| Debian / Ubuntu | `josephine_<version>_amd64.deb`   |
| Fedora / RHEL   | `josephine-<version>.x86_64.rpm`  |
| Arch Linux      | AUR — `packaging/aur/PKGBUILD`    |
| NixOS / Nix     | Flake : `nix run github:systm-d/josephine` |

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb
# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

### Gestionnaires de paquets

**Homebrew** — ce dépôt est un tap (compilation depuis les sources, Rust
requis ; Linux uniquement, `depends_on :linux`) :

```sh
brew tap systm-d/josephine https://github.com/systm-d/josephine
brew install josephine
```

Ou épinglez-la dans un `Brewfile` :

```ruby
tap "systm-d/josephine", "https://github.com/systm-d/josephine"
brew "josephine"
```

**Arch Linux — AUR :** chaque release embarque un `PKGBUILD` prêt à l'emploi
([`packaging/aur/PKGBUILD`](packaging/aur/PKGBUILD)) :

```sh
curl -LO https://github.com/systm-d/josephine/releases/latest/download/PKGBUILD
makepkg -si
```

**Nix / NixOS :** ce dépôt est un flake. Essayez Joséphine sans rien installer :

```sh
nix run github:systm-d/josephine -- status
```

Installez-la dans votre profil :

```sh
nix profile install github:systm-d/josephine
```

Ou câblez-la dans votre système en ajoutant le flake comme entrée :

```nix
# flake.nix
inputs.josephine.url = "github:systm-d/josephine";
```

Sur NixOS, importez le module et activez la veille en arrière-plan (elle tourne
comme service systemd *utilisateur*, comme pour les autres paquets) :

```nix
imports = [ inputs.josephine.nixosModules.default ];
services.josephine.enable = true;
```

Avec Home Manager, le module a la même forme :

```nix
imports = [ inputs.josephine.homeManagerModules.default ];
services.josephine.enable = true;
```

Le paquet est aussi exposé comme `packages.<system>.default` et via
`overlays.default`, si vous préférez l'ajouter vous-même à
`environment.systemPackages`. Comme une installation Nix vit dans un store en
lecture seule, `josephine update` ne s'installera pas tout seul : il vous
renvoie vers votre configuration.

### Depuis les sources

Rust 1.85+ requis.

```sh
cargo install --git https://github.com/systm-d/josephine josephine
```

## Utilisation

```sh
josephine               # résumé rapide (par défaut)
josephine status        # CPU, mémoire, disque, température, systemd, mises à jour
josephine status --oneline  # une ligne compacte pour une barre d'état (Waybar, polybar, tmux)
josephine doctor        # diagnostic complet, et ce qu'il reste à faire
josephine doctor -v     # détaillé : seuils, 10 processus les plus gourmands, intervalles
josephine history       # 24 dernières heures : min/moy/max, sparklines et événements
josephine history --since 3d --check disk --json   # ciblé, lisible par une machine
josephine daemon start  # lancer la veille en arrière-plan
josephine daemon status # état du démon (PID, uptime)
josephine config init --profile server  # configuration de départ selon la machine
josephine config show   # afficher la configuration courante
josephine config edit   # éditer la config dans $EDITOR, puis revalider
josephine report        # rapport texte daté (-o écrit dans un fichier)
josephine report --since 7d  # ajouter un digest des événements de la semaine
josephine clean         # aperçu de l'espace disque récupérable (--apply vide les caches)
josephine explain       # ce que surveille chaque contrôle et comment agir
josephine explain disk  # explication complète d'un seul contrôle
josephine notify test   # envoyer une notification desktop de test
josephine update        # chercher une nouvelle version sur GitHub et l'installer
josephine completions bash  # complétions shell (bash, zsh, fish, …)
josephine man           # la page de manuel, en roff, sur la sortie standard
josephine --version
```

`josephine status` tire son code de sortie du pire contrôle rencontré — `0`
tout va bien, `1` quelque chose à regarder, `2` quelque chose de critique — pour
tomber directement dans un script ou une barre d'état. Associez-le à
`--oneline` pour une ligne unique, ou à `--json` pour la vue complète lisible
par une machine.

Ces trois codes ne veulent jamais dire que *santé*. Si Joséphine ne peut pas
répondre du tout, elle sort hors de cette bande, en suivant `sysexits(3)` : `64`
si la ligne de commande était mal formée, `70` si la commande a tourné et
échoué. Une barre d'état peut donc distinguer une machine critique d'un appel
cassé.

`josephine update` ne touche au réseau que lorsque vous le lancez — jamais en
arrière-plan. Il détecte comment Joséphine a été installée (`.deb`/`.rpm`/…),
télécharge le paquet correspondant, vérifie sa somme de contrôle, et vous laisse
l'étape privilégiée (`sudo`).

Pour que Joséphine continue de veiller après un redémarrage, activez l'unité
systemd **utilisateur** fournie
([`packaging/systemd/josephine.service`](packaging/systemd/josephine.service)) :

```sh
systemctl --user enable --now josephine
```

Pour un digest hebdomadaire tranquille, activez le timer fourni (opt-in) ; il
lance `josephine report --since 7d` et écrit dans le journal
(`journalctl --user -u josephine-report`) :

```sh
systemctl --user enable --now josephine-report.timer
```

`josephine history --json` produit un document stable : `window_hours` et un
`window` affichable, un drapeau `enabled` (faux quand l'historique est coupé,
les listes étant alors vides plutôt qu'absentes), un tableau `metrics` de
`{check, metric, unit, min, avg, max, series}`, et un tableau `events` de
`{check, metric, from, to, value, message, at}`. `series` contient des paquets
moyennés, du plus ancien au plus récent — horaires jusqu'à 48 h, journaliers
au-delà, pour qu'une fenêtre d'une semaine reste lisible au lieu d'entasser 168
points dans une sparkline.

Les installations `.deb`, `.rpm` et tarball embarquent le manuel : `man
josephine` (et `man josephine-doctor`, et ainsi de suite pour chaque
sous-commande) fonctionne dès l'installation. Installée depuis les sources,
générez-le vous-même :

```sh
josephine man --dir ~/.local/share/man/man1
```

La configuration vit dans `~/.config/josephine/config.yaml` (créée au premier
lancement). `josephine config init --profile <laptop|desktop|server>` en écrit
une de départ à la place : `desktop` retire le contrôle batterie, et `server` le
retire aussi, alerte dès la première unité en échec, surveille disque et inodes
à partir de 80 %, garde 90 jours d'historique, et envoie ses alertes dans le
journal plutôt que vers un bureau devant lequel personne n'est assis. Un profil
n'est qu'un point de départ — le fichier est le vôtre ensuite, et son nom n'est
plus jamais relu. Il refuse d'écraser une configuration existante sans
`--force`.
L'historique et l'état du démon vivent sous `~/.local/share/josephine/`.

### Alimenter un tableau de bord, toujours 100 % local

Joséphine peut écrire ses derniers résultats dans un fichier textfile Prometheus,
que le textfile collector de node-exporter ramassera. Désactivé par défaut ;
aucun serveur, aucun port, rien d'elle ne devient joignable depuis le réseau :

```yaml
export:
  prometheus:
    enabled: true
    # Optionnel. Par défaut ~/.local/share/josephine/josephine.prom — pointez
    # le --collector.textfile.directory de node-exporter sur le dossier qui le
    # contient, ou donnez un chemin absolu dans un dossier qu'il lit déjà.
    path: /var/lib/node_exporter/textfile_collector/josephine.prom
```

Le démon réécrit le fichier après chaque contrôle, de façon atomique, pour qu'un
scrape ne l'attrape jamais à moitié écrit. Il expose
`josephine_metric{check,metric,unit}`, `josephine_check_severity{check}`
(0 ok, 1 attention, 2 critique) et `josephine_last_write_timestamp_seconds` —
les mêmes chiffres que `josephine status`, et rien de plus.

L'en-tête de `status` est volontairement sobre. Envie d'une fioriture ? Déposez
n'importe quel art ASCII/Braille dans `~/.config/josephine/banner.txt` : il
apparaît au-dessus du titre, teinté d'un dégradé. Un exemple prêt à l'emploi
vit dans [`resources/banner.txt`](resources/banner.txt).

## Documentation

- [Architecture](docs/ARCHITECTURE.md) · [Développement](docs/DEVELOPMENT.md) · [État actuel](docs/CURRENT_STATE.md) · [Roadmap](docs/ROADMAP.md)
- [Contribuer](CONTRIBUTING.md) · [Conventions](CONVENTIONS.md) · [Code de conduite](CODE_OF_CONDUCT.md) · [Sécurité](SECURITY.md)
- Site : <https://systm-d.github.io/josephine/>
- In English: [README.md](README.md)

## Développement

```sh
cargo build
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour le workflow et la barrière qualité.

Les nouvelles contributions sont bienvenues, et il y a en général quelque chose
de petit à prendre :

- Bonnes premières issues : <https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22>
- Là où de l'aide est souhaitée : <https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22>

## Licence

Sous licence [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à votre choix.
