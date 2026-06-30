+++
title = "Joséphine"
+++

Joséphine watches your Linux machine silently and only speaks up when it helps —
keeping an eye on CPU, memory, disk, temperature and systemd services, and sending
warm, plain-language desktop notifications. 100% local; no data ever leaves your
computer.

<div class="features">
  <div class="feature">
    <h3>🩺 Five checks</h3>
    <p>CPU, memory, disk, temperature, systemd — early warnings before trouble.</p>
  </div>
  <div class="feature">
    <h3>🔔 Kind notifications</h3>
    <p>Warm desktop messages, never alarmist — never ERROR/FATAL/PANIC.</p>
  </div>
  <div class="feature">
    <h3>🔒 Local &amp; private</h3>
    <p>Everything runs on your machine. No cloud, no telemetry.</p>
  </div>
</div>

## See it in action

### A glance at your machine

Run `josephine` for a one-screen summary. Each check gets a bar, a value and a
plain state — `ok`, `attention` or `critique`.

```
$ josephine
✨ Joséphine
Résumé de l'état de votre machine

CHECK             BARRE                 VALEUR      ÉTAT
────────────────────────────────────────────────────────────────────────
CPU               ███░░░░░░░░░░░░░      16.2 %        ok
Mémoire           █████░░░░░░░░░░░      34.0 %        ok
Disque            ███████████████░      91.0 %  attention
Température       ████░░░░░░░░░░░░      48.0 °C        ok
Systemd           ░░░░░░░░░░░░░░░░   0 service        ok

✨ Joséphine a remarqué quelque chose — consultez `josephine doctor`.
```

### A closer look with `doctor`

When something needs attention, `josephine doctor` explains *what* and *why* —
here, which filesystems are filling up.

```
$ josephine doctor
┌──────────────────────────────────────────────────────────────────────┐
│ Disque               alert                                            │
╞══════════════════════════════════════════════════════════════════════╡
│ Indicateur           ███████████████░      91.0 %                     │
│                      / (ext4) : 91.0 % utilisé (228 / 250 Go)         │
│                      /home (ext4) : 64.2 % utilisé (290 / 452 Go)     │
└──────────────────────────────────────────────────────────────────────┘
```

### Notifications you'll actually read

No jargon, no panic. Joséphine speaks like a calm friend — and tells you the
exact command to dig deeper.

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
    <p>Votre mémoire se détend (54 %). Tout le monde peut souffler, moi y compris.</p>
  </div>
</div>

## Install

```sh
cargo install josephine
```

## Usage

```sh
josephine            # quick status
josephine doctor     # detailed diagnostics
josephine daemon start
```

User-facing messages are intentionally in French — that is part of Joséphine's
character.
