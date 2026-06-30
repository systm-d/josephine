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
