+++
title = "Joséphine"

[extra]
tagline = "L'ange gardien de votre ordinateur."
cta = "Voir sur GitHub"
+++

Joséphine veille en silence sur votre machine Linux et ne prend la parole que
lorsque c'est utile — elle surveille **onze signes vitaux** et envoie des
notifications de bureau chaleureuses, en langage clair. 100 % local ; aucune
donnée ne quitte votre ordinateur.

<div class="features">
  <div class="feature">
    <h3>🩺 Onze checks</h3>
    <p>CPU, mémoire, disque, température, systemd, mises à jour, réseau, batterie, inodes, santé disque SMART et incidents noyau — des alertes avant les ennuis.</p>
  </div>
  <div class="feature">
    <h3>🔔 Notifications bienveillantes</h3>
    <p>Des messages de bureau chaleureux, jamais alarmistes — jamais ERROR/FATAL/PANIC.</p>
  </div>
  <div class="feature">
    <h3>🔒 Local &amp; privé</h3>
    <p>Tout s'exécute sur votre machine. Pas de cloud, pas de télémétrie.</p>
  </div>
  <div class="feature">
    <h3>⬆️ Mises à jour faciles</h3>
    <p><code>josephine update</code> récupère la dernière version et l'installe — réseau seulement quand vous le demandez.</p>
  </div>
</div>

## En action

Lancez `josephine` pour un résumé sur un écran. Chaque check affiche une valeur
et un état clair — `OK`, `attention` ou `critique` :

```
$ josephine
✨ Joséphine
Votre ange gardien système
────────────────────────────────────────────────────────────
  🖥️  Utilisation CPU     24%                               [OK]
  📈  Charge système      1.42 (1m) 1.05 (5m) 0.98 (15m)    [OK]
  🧠  Mémoire             60% (18G / 31G)                   [OK]
  💽  Espace disque       21% de « / » (195G / 937G)        [OK]
  🌡️  Température         74°C                              [OK]
  🛡️  Services critiques  Tous les services fonctionnent    [OK]
  🔄  Mises à jour        30 mises à jour disponibles       [!] ATTENTION
  🌐  Réseau              10 ms (passerelle)                [OK]
  🔋  Batterie            99 % (branchée)                   [OK]
  🗂️  Inodes              4% de « / »                       [OK]
  🐧  Noyau               0 incident (1 h)                  [OK]
```

`josephine doctor` détaille ensuite, check par check, avec des barres par métrique
et les processus les plus actifs ; `josephine history` montre les min/moy/max sur
24 h avec des tendances en sparklines.

### Des notifications qu'on lit vraiment

Ni jargon, ni panique. Joséphine parle comme une amie posée — et vous indique la
commande exacte pour creuser.

<div class="notifs">
  <div class="notif notif--warn">
    <span class="notif__icon">✨</span>
    <p>Votre disque est à 91 % — il tousse un peu. Même au paradis, on n'a pas de
    stockage illimité. Je peux vous aider à voir ce qui encombre :
    <code>josephine doctor</code>.</p>
  </div>
  <div class="notif notif--crit">
    <span class="notif__icon">✨</span>
    <p>Mon cher… 97 % de CPU. Votre machine court plus vite que moi avec mes ailes —
    et ce n'est pas un compliment. <code>josephine doctor</code>, vite.</p>
  </div>
  <div class="notif notif--ok">
    <span class="notif__icon">✨</span>
    <p>Votre batterie a repris des forces (ou vous voilà branché). Ouf — je respire
    mieux, moi aussi.</p>
  </div>
</div>

## Installation

Récupérez un paquet depuis la [dernière release](https://github.com/systm-d/josephine/releases/latest) :

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Ou compilez depuis les sources (Rust 1.85+) : `cargo install --git https://github.com/systm-d/josephine josephine`.

## Utilisation

```sh
josephine            # état rapide
josephine doctor     # diagnostic détaillé
josephine history    # tendances 24 h (min/moy/max + sparklines)
josephine update     # vérifie et installe une nouvelle version
josephine daemon start
```

> Joséphine parle **français** — cela fait partie de son caractère. Une option de
> langue anglais / français est prévue dans la roadmap.
