+++
title = "Joséphine"

[extra]
eyebrow = "Local Linux guardian"
tagline = "Your machine, watched over — quietly."
lede = "She keeps an eye on fourteen vital signs of your Linux machine and speaks up only when something needs you. Direct, calm, and entirely local — no cloud, ever."
cta = "View on GitHub"
cta2 = "Install"
readout_alt = "Example josephine status output: fourteen checks, one flagged for attention"
+++

<section>
<p class="eyebrow">What she watches</p>

## Fourteen vital signs

<p class="section__lede">Each check is independent, configurable, and read straight from the kernel and <code>/sys</code> — nothing ever leaves the machine.</p>

<div class="signs">
  <div class="sign"><h3><span class="mark">✦</span> Fourteen checks</h3><p>CPU, memory, disk, temperature, systemd, updates, network, battery, inodes, SMART, kernel, filesystem, clock &amp; security.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> Direct alerts</h3><p>Plain-language desktop notes, calm and to the point — never <code>ERROR</code> / <code>FATAL</code> / <code>PANIC</code>.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> Fully local</h3><p>Everything runs on your machine. No cloud, no telemetry, no account.</p></div>
  <div class="sign"><h3><span class="mark">✦</span> Machine-readable</h3><p><code>--json</code> for scripting, shell completions, and a self-update from GitHub releases.</p></div>
</div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">See it in action</p>

## One screen, at a glance

<p class="section__lede">Severity is carried by shape <em>and</em> colour — <span class="g-ok">●</span> ok, <span class="g-warn">▲</span> attention, <span class="g-crit">✕</span> critical — so it reads even in a pipe (<code>[ok] [!] [x]</code> off a terminal). <code>josephine doctor</code> explains it check by check; <code>josephine history</code> shows 24-hour min / avg / max with sparkline trends <code>▁▂▄▇▅▃</code>.</p>

<div class="term"><div class="term__bar"><span class="term__dot"></span><span class="term__dot"></span><span class="term__dot"></span><span class="term__title">josephine doctor</span></div><pre><span class="dim">✦ Joséphine · diagnostic                        14:40</span>
14 checks · 1 to look at
<span class="rule">──────────────────────────────────────────────────</span>
 <span class="g-warn">▲</span>  Updates · <span class="g-warn">attention</span>                    30 available
    ▓▓▓▓▓▓▓░░░░░░░  apt: 30 package updates pending
 <span class="g-ok">●</span>  Disk · <span class="g-ok">ok</span>                             21 %
    ▓▓▓░░░░░░░░░░░  « / » (btrfs) 195G / 937G · SSD
 <span class="g-ok">●</span>  Kernel · <span class="g-ok">ok</span>                       0 incidents
    No kernel incidents in the last hour.</pre></div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">Notifications</p>

## She speaks only when it helps

<p class="section__lede">No jargon, no theatrics — a calm, direct line, and always the exact command to dig deeper.</p>

<div class="notifs">
  <div class="notif notif--warn"><span class="notif__glyph g-warn">▲</span><p>Disk at 91% on <code>/</code>. Storage is filling up — <code>josephine doctor</code> shows what's taking the room.</p></div>
  <div class="notif notif--crit"><span class="notif__glyph g-crit">✕</span><p>CPU at 97%, and it's holding there. Something is pinning it — <code>josephine doctor</code> will point you to it.</p></div>
  <div class="notif notif--ok"><span class="notif__glyph g-ok">●</span><p>Battery is back to a healthy level (or you're plugged in). All clear.</p></div>
</div>
</section>

<hr class="divider" />
<section>
<p class="eyebrow">Commands</p>

## A small, honest toolbox

<ul class="commands">
  <li><code>josephine</code><span>a one-screen summary of every check</span></li>
  <li><code>josephine doctor</code><span>detailed diagnostics, check by check (<code>-v</code> for more)</span></li>
  <li><code>josephine history</code><span>24-hour min / avg / max with sparkline trends</span></li>
  <li><code>josephine report</code><span>a dated, plain-text health report (<code>--json</code> too)</span></li>
  <li><code>josephine clean</code><span>preview reclaimable disk space (<code>--apply</code> to clear caches)</span></li>
  <li><code>josephine fix</code><span>guided remediation for failed services / low disk</span></li>
  <li><code>josephine explain</code><span>what each check watches and how to act</span></li>
  <li><code>josephine completions</code><span>shell completions for bash, zsh, fish</span></li>
  <li><code>josephine daemon start</code><span>run the background watcher</span></li>
</ul>

Deeper docs live in the repository — [Architecture](https://github.com/systm-d/josephine/blob/main/docs/ARCHITECTURE.md) · [Current state](https://github.com/systm-d/josephine/blob/main/docs/CURRENT_STATE.md) · [Roadmap](https://github.com/systm-d/josephine/blob/main/docs/ROADMAP.md). Your configuration lives at `~/.config/josephine/config.yaml` (created on first run), and history under `~/.local/share/josephine/`.
</section>

<hr class="divider" />
<section id="install">
<p class="eyebrow">Install</p>

## Up in a minute

Grab a package from the [latest release](https://github.com/systm-d/josephine/releases/latest):

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Prefer to build it yourself? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+). On Linux with Homebrew, `brew install …/josephine.rb` builds from source.

To keep Joséphine watching across reboots, enable the bundled systemd **user** unit:

```sh
systemctl --user enable --now josephine
```

<p class="callout"><strong>Joséphine is a guardian, not a dashboard.</strong> She stays out of your way, keeps her voice calm, and only speaks when it helps — for people who'd rather their computer simply took care of itself.</p>

> Joséphine speaks English by default — set `language: fr` in the config for her French voice. The warm, direct tone is preserved in both.
</section>
