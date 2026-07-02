+++
title = "Joséphine"

[extra]
tagline = "Your computer's quiet guardian angel."
lede = "She keeps a gentle eye on your Linux machine — eleven vital signs, watched in silence — and only whispers when something truly needs you. 100% local. No cloud, ever."
cta = "View on GitHub"
cta2 = "Install"
+++

<div class="features">
  <div class="feature">
    <span class="feature__icon">🩺</span>
    <h3>Eleven checks</h3>
    <p>CPU, memory, disk, temperature, systemd, updates, network, battery, inodes, SMART &amp; kernel.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">🔔</span>
    <h3>Kind alerts</h3>
    <p>Warm, plain-language desktop notes — never ERROR / FATAL / PANIC.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">🔒</span>
    <h3>Fully local</h3>
    <p>Everything runs on your machine. No cloud, no telemetry, no account.</p>
  </div>
  <div class="feature">
    <span class="feature__icon">⬆️</span>
    <h3>Self-update</h3>
    <p><code>josephine update</code> fetches &amp; installs the latest release, on request.</p>
  </div>
</div>

## 👀 See it in action

Run `josephine` for a one-screen summary. Each check shows a value and a plain
state — `OK`, `attention` or `critique`:

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

`josephine doctor` explains it check by check, with per-metric bars and the top
processes. `josephine history` shows 24-hour **min / avg / max** with sparkline
trends: `▁▂▄▇▅▃`.

### 💬 Notifications you'll actually read

No jargon, no panic. Joséphine speaks like a calm friend — and always tells you
the exact command to dig deeper.

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

## 📖 Commands

<ul class="commands">
  <li><code>josephine</code><span>a one-screen summary of every check</span></li>
  <li><code>josephine doctor</code><span>detailed diagnostics, check by check (<code>-v</code> for more)</span></li>
  <li><code>josephine history</code><span>24-hour min/avg/max with sparkline trends</span></li>
  <li><code>josephine report</code><span>a dated, plain-text health report (<code>-o</code> to a file)</span></li>
  <li><code>josephine clean</code><span>preview reclaimable disk space (<code>--apply</code> to clear caches)</span></li>
  <li><code>josephine fix</code><span>guided remediation for failed services / low disk</span></li>
  <li><code>josephine update</code><span>check for &amp; install a newer version</span></li>
  <li><code>josephine daemon start</code><span>run the background watcher</span></li>
  <li><code>josephine config edit</code><span>open the config in <code>$EDITOR</code>, then re-validate</span></li>
</ul>

Deeper docs live in the repository — [Architecture](https://github.com/systm-d/josephine/blob/main/docs/ARCHITECTURE.md) ·
[Current state](https://github.com/systm-d/josephine/blob/main/docs/CURRENT_STATE.md) ·
[Roadmap](https://github.com/systm-d/josephine/blob/main/docs/ROADMAP.md). Your
configuration lives at `~/.config/josephine/config.yaml` (created on first run),
and history under `~/.local/share/josephine/`.

## 🕊️ Install {#install}

Grab a package from the [latest release](https://github.com/systm-d/josephine/releases/latest):

```
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Prefer to build it yourself? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+).

To keep Joséphine watching across reboots, enable the bundled systemd **user** unit:

```
systemctl --user enable --now josephine
```

<p class="callout">✨ <strong>Joséphine is a guardian angel, not a dashboard.</strong>
She stays out of your way, keeps her voice warm, and only speaks when it helps —
<em>made with ♥ for people who'd rather their computer simply took care of
itself.</em></p>

> The screenshots show Joséphine's French voice — part of her character. An
> English / French language option is on the way.
