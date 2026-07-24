+++
title = "Joséphine"

[extra]
eyebrow = "Gardien Linux local"
tagline = "Votre machine, sous bonne garde — en silence."
lede = "Vous avez déjà htop, smartctl et journalctl. Joséphine, elle, veille entre deux coups d'œil — elle repère les problèmes lents (un disque qui se remplit, un SSD qui faiblit, un service arrêté en silence) et vous prévient avant qu'ils ne deviennent les vôtres."
cta = "Voir sur GitHub"
cta2 = "Installer"
readout_alt = "Exemple de sortie josephine status : quatorze contrôles, un à surveiller"
+++

<section id="install" class="reveal">
<p class="eyebrow">Installer</p>

## En place en une minute

Récupérez un paquet depuis la [dernière release](https://github.com/systm-d/josephine/releases/latest) :

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Vous préférez la compiler vous-même ? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+). Sous Linux avec Homebrew, `brew tap systm-d/josephine https://github.com/systm-d/josephine && brew install josephine` compile depuis les sources.

Pour que Joséphine veille au fil des redémarrages, activez l'unité systemd **user** fournie :

```sh
systemctl --user enable --now josephine
```

<p class="callout"><strong>Joséphine est un ange gardien, pas un tableau de bord.</strong> Elle reste discrète, garde une voix calme, et ne parle que lorsque ça aide — pour celles et ceux qui préfèrent que leur ordinateur s'occupe simplement de lui-même.</p>

> Joséphine parle anglais par défaut — mettez `language: fr` dans la config pour sa voix française. Le ton chaleureux et direct est préservé dans les deux.
</section>
