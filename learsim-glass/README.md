# learsim-glass

Kiosk **display** app for [glassout](https://glassout.flyingart.dev) views.
Runs fullscreen on a Raspberry Pi (no desktop environment required) and on
Windows / macOS. One machine = one **device**; each physical monitor = one
**screen**; each screen shows exactly one **view**.

Its sibling, [`learsim-glass-admin`](../learsim-glass-admin), configures which
view each screen shows — over the LAN, so you can drive a headless Pi from a
laptop.

---

## How it works

```
                 learsim-glass-admin (laptop)
                          │  HTTP (control)
                          ▼
   ┌──────────────────────────────────────────┐
   │  learsim-glass  (Raspberry Pi / PC)        │
   │                                            │
   │   control server  ── 0.0.0.0:8770          │  ← admin talks here
   │        │                                   │
   │        ├─ screen-1  ─ fullscreen window ─┐ │
   │        └─ screen-2  ─ fullscreen window ─┤ │
   │                                          │ │
   └──────────────────────────────────────────┘ │
              each window shows a view:          │
              • local  (standby / clock)  ───────┘  rendered by the app
              • glassout ── navigates to ──►  http://<engine>:8787/panel/<id>
```

- On first launch the app **detects your monitors and creates one screen per
  monitor**, so a Pi with two displays gets two independent configs out of the
  box. Each screen opens a decoration-less fullscreen window pinned to its
  monitor.
- The embedded **control server** listens on `0.0.0.0:8770`. The admin app uses
  it to list screens and reassign views. Changes persist and apply **live** —
  no restart.
- A **glassout** view is a live MSFS panel. Per the glassout SDK docs, the
  browser path is simply the engine's viewer URL, so the window navigates
  straight to `http://<engine>:8787/panel/<PanelId>?fps=…&fit=…`. Top-level
  navigation (not an iframe) avoids the mixed-content block that would hit a LAN
  `http://` engine from a secure webview origin.
- **Local** views (standby, clock) are rendered by the app itself and double as
  a fallback.

> **Ports:** the control server uses **8770** on purpose — the glassout engine
> owns **8787**, so both can run on the same machine.

---

## Develop

Prerequisites: [Rust](https://rustup.rs), Node 18+, and — on Linux — the Tauri
system deps (`webkit2gtk`, `libsoup`, etc; see
<https://tauri.app/start/prerequisites/>).

```bash
npm install
npm run tauri dev
```

Frontend stack: **TypeScript + Vite + Tailwind CSS v4** (design tokens live in
`src/styles.css` under `@theme`). Tooling: **ESLint** (`npm run lint`) and
**Prettier** with the Tailwind class-sorting plugin (`npm run format`).

## Build

```bash
npm install
npm run tauri build          # binary in src-tauri/target/release/
```

Bundling installers is disabled by default (`bundle.active = false`) so the
build doesn't require icon assets. To produce installers, generate icons
(`npm run tauri icon path/to/icon.png`) and set `bundle.active = true` in
`src-tauri/tauri.conf.json`.

---

## Configuration

Stored as JSON in the OS config dir (`app_config_dir`):

- Linux: `~/.config/dev.learsim.glass/config.json`
- Windows: `%APPDATA%\dev.learsim.glass\config.json`
- macOS: `~/Library/Application Support/dev.learsim.glass/config.json`

```jsonc
{
  "device": { "id": "…uuid…", "name": "cockpit-pi" },
  "port": 8770,
  "screens": [
    { "id": "screen-1", "name": "Screen 1", "monitor": 0,
      "viewId": "standby", "settings": {} },
    { "id": "screen-2", "name": "Screen 2", "monitor": 1,
      "viewId": "glassout",
      "settings": { "engineUrl": "http://192.168.1.50:8787",
                    "panelId": "PFD_Captain", "fit": "stretch",
                    "targetFps": "30", "debug": false } }
  ]
}
```

You normally never edit this by hand — use the admin app. It's here so you can
see/version the state.

### Views

| id            | what it shows                                                  |
|---------------|----------------------------------------------------------------|
| `standby`     | Device + screen name (optional custom `message`).              |
| `clock`       | Large clock (`hourFormat`, `showSeconds`, `showDate`, `utc`).  |
| `testpattern` | Alignment grid, edge frame, centre cross, and colour bars for setting up physical panels/bezels. |
| `glassout`    | A live MSFS panel from a glassout engine (see below).          |

### Self-healing glassout screens

A glassout screen's window is driven by a background monitor that probes the
engine's `GET /status` every few seconds:

- **Engine reachable** → the window shows the panel viewer URL.
- **Engine down / rebooting / not yet configured** → the window shows a branded
  "connecting to glassout…" placeholder instead of a browser error page, and
  **recovers automatically** the moment the engine answers again.

So if the sim PC reboots, the cockpit screens ride it out and come back on their
own — no touch, no restart.

Adding a local view = add a descriptor in `src-tauri/src/views.rs` and a
renderer in `src/views/`. Both are keyed by the same id.

### glassout view settings

| key         | meaning                                                        |
|-------------|----------------------------------------------------------------|
| `engineUrl` | Engine base URL, e.g. `http://192.168.1.50:8787`.              |
| `panelId`   | Panel to show, e.g. `PFD_Captain`.                            |
| `fit`       | `contain` (letterbox) / `stretch` / `native`.                 |
| `targetFps` | Per-viewer FPS cap, 10–120.                                    |
| `clickDelay`| Hover delay (ms, 0–5000) before a touch click fires. Blank → engine default (300). Use `0` for gauges, `300–500` for touch instruments (e.g. G3000 GTC). |
| `debug`     | Enable the engine's latency overlay.                          |
| `path`      | Advanced: a ready-made viewer path (`/canvas?…`, `/instance/…`) that overrides the panel fields. |

URL construction lives in one place: `src-tauri/src/glassout.rs`
(`build_view_url`). It mirrors the engine's `/panel/{id}` viewer semantics.
Panel **discovery** (picking from a list) is the admin app's job, done over the
engine's plain HTTP `GET /status` endpoint — no SDK required.

---

## Running headless on a Raspberry Pi (no desktop environment)

You don't need a desktop environment — just a minimal surface for the webview.

### Option A — `cage` (Wayland kiosk, recommended)

`cage` is a single-app kiosk compositor that runs directly on DRM/KMS.

```bash
sudo apt install cage
```

`/etc/systemd/system/learsim-glass.service`:

```ini
[Unit]
Description=learsim-glass kiosk
After=systemd-user-sessions.service

[Service]
User=pi
# DRM/KMS seat access
TTYPath=/dev/tty1
PAMName=login
Environment=XDG_RUNTIME_DIR=/run/user/1000
ExecStart=/usr/bin/cage -- /home/pi/learsim-glass
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now learsim-glass
```

### Option B — bare X11

```bash
sudo apt install xserver-xorg xinit openbox
# ~/.xinitrc:  exec /home/pi/learsim-glass
startx
```

### Raspberry Pi notes

- **Two screens need two outputs.** A **Pi 4 (dual micro-HDMI) or Pi 5** can
  drive two monitors → two auto-provisioned screens. A **Pi 3 has a single
  HDMI** and will show one screen.
- **glassout panels are live video.** webkit2gtk decoding is heavy on a Pi 3 —
  fine for `standby`/`clock`, sluggish for glassout. Keep `targetFps` low
  (10–15) on a Pi 3; use a Pi 4/5 for smooth panels.
- **Building on a Pi 3 is slow** (1 GB RAM) — build on a Pi 4/5, cross-compile,
  or add swap.
- Enable KMS (`dtoverlay=vc4-kms-v3d` in `/boot/config.txt`) for GPU-accelerated
  compositing under `cage`.
