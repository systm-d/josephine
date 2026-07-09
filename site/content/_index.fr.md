+++
title = "Joséphine"

[extra]
eyebrow = "Gardien Linux local"
tagline = "Votre machine, sous bonne garde — en silence."
lede = "Elle veille sur quatorze signes vitaux de votre machine Linux et ne parle que lorsque quelque chose vous concerne. Directe, calme, et 100 % locale — jamais de cloud."
cta = "Voir sur GitHub"
cta2 = "Installer"
readout_alt = "Exemple de sortie josephine status : quatorze contrôles, un à surveiller"
+++

<section>
<p class="eyebrow">Ce qu'elle surveille</p>

## Quatorze signes vitaux

<p class="section__lede">Chaque contrôle est indépendant, configurable, et lu directement dans le noyau et <code>/sys</code> — rien ne quitte jamais la machine.</p>

<div class="signs">
  <div class="sign"><h3><span class="mark">✦</span> Quatorze contrôles</h3><p>CPU, mémoire, disque, température, systemd, mises à jour, réseau, batterie, inodes, SMART, noyau, système de fichiers, horloge &amp; sécurité.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> Alertes directes</h3><p>Des notes desktop en clair, calmes et précises — jamais <code>ERROR</code> / <code>FATAL</code> / <code>PANIC</code>.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> 100 % locale</h3><p>Tout tourne sur votre machine. Pas de cloud, pas de télémétrie, pas de compte.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> Lisible par machine</h3><p><code>--json</code> pour le scripting, complétions shell, et mise à jour depuis les releases GitHub.</p></div>
</div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">En action</p>

## Un écran, d'un coup d'œil

<p class="section__lede">La gravité est portée par la forme <em>et</em> la couleur — <span class="g-ok">●</span> ok, <span class="g-warn">▲</span> attention, <span class="g-crit">✕</span> critique — lisible même dans un pipe (<code>[ok] [!] [x]</code> hors terminal). <code>josephine doctor</code> détaille contrôle par contrôle ; <code>josephine history</code> montre les min / moy / max sur 24 h avec des sparklines <code>▁▂▄▇▅▃</code>.</p>

<div class="term"><div class="term__bar"><span class="term__dot"></span><span class="term__dot"></span><span class="term__dot"></span><span class="term__title">josephine doctor</span></div><pre><span class="dim">✦ Joséphine · diagnostic                        14:40</span>
14 contrôles · 1 à regarder
<span class="rule">──────────────────────────────────────────────────</span>
 <span class="g-warn">▲</span>  Mises à jour · <span class="g-warn">attention</span>             30 dispo.
    ▓▓▓▓▓▓▓░░░░░░░  apt : 30 paquets à mettre à jour
 <span class="g-ok">●</span>  Disque · <span class="g-ok">ok</span>                           21 %
    ▓▓▓░░░░░░░░░░░  « / » (btrfs) 195G / 937G · SSD
 <span class="g-ok">●</span>  Noyau · <span class="g-ok">ok</span>                        0 incident
    Aucun incident noyau sur la dernière heure.</pre></div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">Notifications</p>

## Elle ne parle que si ça aide

<p class="section__lede">Ni jargon, ni dramatisation — une ligne calme et directe, et toujours la commande exacte pour creuser.</p>

<div class="notifs">
  <div class="notif notif--warn"><span class="notif__glyph g-warn">▲</span><p>Disque à 91 % sur <code>/</code>. L'espace se remplit — <code>josephine doctor</code> montre ce qui prend la place.</p></div>
  <div class="notif notif--crit"><span class="notif__glyph g-crit">✕</span><p>CPU à 97 %, et ça tient. Quelque chose le sature — <code>josephine doctor</code> vous mènera à la cause.</p></div>
  <div class="notif notif--ok"><span class="notif__glyph g-ok">●</span><p>La batterie est revenue à un niveau sain (ou vous êtes branché). Tout est au vert.</p></div>
</div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">Commandes</p>

## Une petite boîte à outils honnête

<ul class="commands">
  <li><code>josephine</code><span>un résumé d'un écran de chaque contrôle</span></li>
  <li><code>josephine doctor</code><span>diagnostic détaillé, contrôle par contrôle (<code>-v</code> pour plus)</span></li>
  <li><code>josephine history</code><span>min / moy / max sur 24 h avec sparklines</span></li>
  <li><code>josephine report</code><span>un rapport texte daté (<code>--json</code> aussi)</span></li>
  <li><code>josephine clean</code><span>aperçu de l'espace disque récupérable (<code>--apply</code> pour vider les caches)</span></li>
  <li><code>josephine fix</code><span>remédiation guidée pour services en échec / disque plein</span></li>
  <li><code>josephine explain</code><span>ce que chaque check surveille et comment agir</span></li>
  <li><code>josephine completions</code><span>complétions shell pour bash, zsh, fish</span></li>
  <li><code>josephine daemon start</code><span>lance le veilleur en arrière-plan</span></li>
</ul>

La doc détaillée vit dans le dépôt — [Architecture](https://github.com/systm-d/josephine/blob/main/docs/ARCHITECTURE.md) · [État actuel](https://github.com/systm-d/josephine/blob/main/docs/CURRENT_STATE.md) · [Roadmap](https://github.com/systm-d/josephine/blob/main/docs/ROADMAP.md). Votre configuration vit dans `~/.config/josephine/config.yaml` (créée au premier lancement), et l'historique sous `~/.local/share/josephine/`.
</section>

<hr class="divider" />
<section id="install">
<p class="eyebrow">Installer</p>

## En place en une minute

Récupérez un paquet depuis la [dernière release](https://github.com/systm-d/josephine/releases/latest) :

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Vous préférez la compiler vous-même ? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+). Sous Linux avec Homebrew, `brew install …/josephine.rb` compile depuis les sources.

Pour que Joséphine veille au fil des redémarrages, activez l'unité systemd **user** fournie :

```sh
systemctl --user enable --now josephine
```

<p class="callout"><strong>Joséphine est un ange gardien, pas un tableau de bord.</strong> Elle reste discrète, garde une voix calme, et ne parle que lorsque ça aide — pour celles et ceux qui préfèrent que leur ordinateur s'occupe simplement de lui-même.</p>

> Joséphine parle anglais par défaut — mettez <code>language: fr</code> dans la config pour sa voix française. Le ton chaleureux et direct est préservé dans les deux.
</section>
