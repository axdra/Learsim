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

> **Always launch through the Tauri CLI** (`npm run tauri dev`) from *this*
> folder — not `cargo run` / your IDE's Run button inside `src-tauri`. A debug
> build points the webview at the Vite dev server (`http://localhost:1420`);
> running cargo alone never starts Vite, so the window loads nothing and shows a
> **blank screen**. Same for a standalone binary: build it with
> `npm run tauri build`, which builds the frontend into `../dist` and embeds it.

### Troubleshooting a blank / white screen

1. **Launched via cargo/IDE instead of the Tauri CLI** → the Vite dev server
   isn't running. Use `npm run tauri dev` (or `pnpm tauri dev`).
2. **Frontend deps not installed** (after the Tailwind change) → run
   `npm install` in this folder first.
3. Otherwise open the webview devtools (right-click → Inspect in a dev build)
   and check the Console/Network tabs. The app now also prints uncaught errors
   directly onto the screen instead of failing to white.

## Build

```bash
npm install
npm run tauri build          # binary in src-tauri/target/release/
```

A placeholder icon set is included under `src-tauri/icons/` (needed by
`tauri-build` for the Windows resource on every build). Installer bundling is
disabled by default (`bundle.active = false`); to produce installers, set it to
`true` in `src-tauri/tauri.conf.json` (and install the platform bundler tools).
Replace the icons any time with `npm run tauri icon path/to/icon.png`.

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

### Building on a Raspberry Pi (`rustc` SIGSEGV)

`rustc` crashing with `SIGSEGV` while compiling trivial crates (`unicode-ident`,
`proc-macro2`, `serde`) is an environment problem, not a code one. In order:

1. **16 KB memory pages (most common cause).** 64-bit Raspberry Pi OS may run a
   kernel with 16 KB pages, but the prebuilt `rustc` assumes 4 KB pages and
   segfaults deep in `librustc_driver`. Check with `getconf PAGESIZE` — if it
   prints `16384`, force the 4 KB-page kernel by adding to
   `/boot/firmware/config.txt` (or `/boot/config.txt` on older OSes):
   ```
   kernel=kernel8.img
   ```
   then reboot and re-check (`getconf PAGESIZE` should print `4096`).
2. **Undervoltage / heat.** `vcgencmd get_throttled` — anything but `0x0` means
   the PSU or cooling is inadequate, which causes random SIGSEGVs. Use a 5V/3A
   supply and a heatsink.
3. **Low RAM (Pi 3 = 1 GB).** Add swap (default is ~100 MB) and compile
   single-threaded:
   ```bash
   sudo dphys-swapfile swapoff
   sudo sed -i 's/^CONF_SWAPSIZE=.*/CONF_SWAPSIZE=2048/' /etc/dphys-swapfile
   sudo dphys-swapfile setup && sudo dphys-swapfile swapon
   CARGO_BUILD_JOBS=1 RUST_MIN_STACK=16777216 npm run tauri build
   ```
4. Still crashing on trivial crates → the toolchain is likely corrupted (often
   from a prior OOM-killed build):
   `rustup toolchain uninstall stable && rustup toolchain install stable`.

**Easiest of all:** build on a Pi 4/5 or cross-compile on a PC, then copy the
binary + `dist/` to the Pi 3 and run it — the Pi 3 struggles to *compile* Tauri
even though it runs the display fine.

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
