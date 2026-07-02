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
Your system's guardian angel
────────────────────────────────────────────────────────────
  🖥️  CPU usage           24%                               [OK]
  🧠  Memory              60% (18G / 31G)                   [OK]
  💽  Disk space          21% of “/” (195G / 937G)          [OK]
  🌡️  Temperature         74°C                              [OK]
  🛡️  Critical services   All services running              [OK]
  🔄  Updates             30 updates available              [!] WARNING
  🌐  Network             10 ms (gateway)                   [OK]
  🔋  Battery             99 % (plugged in)                 [OK]
  🗂️  Inodes              4% of “/”                          [OK]
  🐧  Kernel              0 incidents (1 h)                 [OK]
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
    <p>Your disk is at 91% — it's coughing a little. Even in heaven, storage
    isn't unlimited. I can help you see what's piling up:
    <code>josephine doctor</code>.</p>
  </div>
  <div class="notif notif--crit">
    <span class="notif__icon">✨</span>
    <p>Goodness… 97% CPU. Your machine is running faster than I can flap my
    wings — and that's no compliment. <code>josephine doctor</code>, quick.</p>
  </div>
  <div class="notif notif--ok">
    <span class="notif__icon">✨</span>
    <p>Your battery is back to strength (or you're plugged in). Phew — I breathe
    easier too.</p>
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

> Joséphine speaks **English by default** — set `language: fr` in the config for
> her French voice. The warm guardian-angel tone is preserved in both.
