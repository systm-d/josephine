+++
title = "Joséphine"

[extra]
tagline = "L'ange gardien de votre ordinateur."
lede = "Elle veille en douceur sur votre machine Linux — onze signes vitaux, surveillés en silence — et ne murmure que lorsque quelque chose a vraiment besoin de vous. 100 % local. Jamais de cloud."
cta = "Voir sur GitHub"
cta2 = "Installer"
+++

<div class="features">
  <div class="feature">
    <span class="feature__icon">🩺</span>
    <h3>Onze checks</h3>
    <p>CPU, mémoire, disque, température, systemd, mises à jour, réseau, batterie, inodes, SMART &amp; noyau.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">🔔</span>
    <h3>Alertes douces</h3>
    <p>Des notifications chaleureuses, en langage clair — jamais ERROR / FATAL / PANIC.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">🔒</span>
    <h3>100 % local</h3>
    <p>Tout s'exécute sur votre machine. Pas de cloud, pas de télémétrie, pas de compte.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">⬆️</span>
    <h3>Mise à jour</h3>
    <p><code>josephine update</code> récupère &amp; installe la dernière version, sur demande.</p>
  </div>
</div>

## 👀 En action

Lancez `josephine` pour un résumé sur un écran. Chaque check affiche une valeur
et un état clair — `OK`, `attention` ou `critique` :

```
$ josephine
✨ Joséphine
Votre ange gardien système
────────────────────────────────────────────────────────────
  🖥️  Utilisation CPU     24%                               [OK]
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

`josephine doctor` détaille check par check, avec des barres par métrique et les
processus les plus actifs. `josephine history` montre les **min / moy / max** sur
24 h avec des tendances en sparklines : `▁▂▄▇▅▃`.

### 💬 Des notifications qu'on lit vraiment

Ni jargon, ni panique. Joséphine parle comme une amie posée — et vous indique
toujours la commande exacte pour creuser.

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

## 📖 Commandes

<ul class="commands">
  <li><code>josephine</code><span>un résumé de tous les checks sur un écran</span></li>
  <li><code>josephine doctor</code><span>diagnostic détaillé, check par check (<code>-v</code> pour plus)</span></li>
  <li><code>josephine history</code><span>min/moy/max sur 24 h avec tendances en sparklines</span></li>
  <li><code>josephine report</code><span>un rapport système daté, en texte (<code>-o</code> vers un fichier)</span></li>
  <li><code>josephine clean</code><span>aperçu de l'espace récupérable (<code>--apply</code> pour nettoyer)</span></li>
  <li><code>josephine fix</code><span>corrections guidées : services en échec / disque serré</span></li>
  <li><code>josephine update</code><span>vérifie &amp; installe une nouvelle version</span></li>
  <li><code>josephine daemon start</code><span>lance la surveillance en arrière-plan</span></li>
  <li><code>josephine config edit</code><span>ouvre la config dans <code>$EDITOR</code>, puis revalide</span></li>
</ul>

La doc complète vit dans le dépôt — [Architecture](https://github.com/systm-d/josephine/blob/main/docs/ARCHITECTURE.md) ·
[État actuel](https://github.com/systm-d/josephine/blob/main/docs/CURRENT_STATE.md) ·
[Roadmap](https://github.com/systm-d/josephine/blob/main/docs/ROADMAP.md). Votre
configuration vit dans `~/.config/josephine/config.yaml` (créé au premier
lancement), et l'historique sous `~/.local/share/josephine/`.

## 🕊️ Installation {#install}

Récupérez un paquet depuis la [dernière release](https://github.com/systm-d/josephine/releases/latest) :

```
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Vous préférez compiler ? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+).

Pour que Joséphine veille au fil des redémarrages, activez l'unité systemd **user** fournie :

```
systemctl --user enable --now josephine
```

<p class="callout">✨ <strong>Joséphine est un ange gardien, pas un tableau de bord.</strong>
Elle reste discrète, garde une voix chaleureuse et ne parle que lorsque c'est
utile — <em>faite avec ♥ pour celles et ceux qui préfèrent que leur ordinateur
prenne soin de lui-même.</em></p>

> Les captures montrent la voix française de Joséphine — cela fait partie de son
> caractère. Une option de langue anglais / français arrive bientôt.
