# learsim-glass-native

A **native, no-webview** glass display client. It renders glassout MSFS panels
and the local views (standby/clock/testpattern) directly with **SDL2 on KMSDRM**
— straight to the display, **no browser engine, no X, no Wayland, no compositor**.

Built to fix the Raspberry Pi pain points of the webview build:
- **Fast & light** — decodes the engine's JPEG frame stream with `zune-jpeg`
  and blits to the screen. No webkit2gtk.
- **No compositor headaches** — runs on the bare console via KMSDRM, so the
  fullscreen/cursor/scaling problems simply don't exist.
- **Touch** — taps are mapped to panel coordinates and sent to the engine as
  `panel.input`, so touch instruments (G3000 GTC etc.) work.

It exposes the **same control server (`:8770`) and config** as the Tauri client,
so the **admin app manages it identically** — no admin changes needed. Use this
on the Pi; keep the Tauri build for Windows/macOS or as a fallback.

## Get it (GitHub Actions)
`.github/workflows/build-pi.yml` has a **`native`** job that builds it for arm64
in a Debian Bookworm container. Run the workflow, then download the
`learsim-glass-native-pi-arm64` artifact — a single self-contained binary.

## Run on the Pi
Install the runtime libraries and a font, and give your user DRM access:
```bash
sudo apt install -y libsdl2-2.0-0 libsdl2-ttf-2.0-0 fonts-dejavu-core
sudo usermod -aG video,render "$USER"      # DRM/KMS access (log out/in after)
mkdir -p ~/glass && cp learsim-glass-native ~/glass/ && chmod +x ~/glass/learsim-glass-native
```

Quick test on the Pi's console (tty1, not SSH):
```bash
SDL_VIDEODRIVER=kmsdrm ~/glass/learsim-glass-native
```

Auto-start service — `/etc/systemd/system/learsim-glass.service`:
```ini
[Unit]
Description=learsim-glass-native kiosk
After=systemd-user-sessions.service network-online.target

[Service]
User=learsim
TTYPath=/dev/tty1
PAMName=login
Environment=SDL_VIDEODRIVER=kmsdrm
ExecStart=/home/learsim/glass/learsim-glass-native
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
```
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now learsim-glass
curl -s http://localhost:8770/api/health; echo    # {"app":"learsim-glass-native",...}
```
If the screen stays on the text console, free tty1: `sudo systemctl disable --now getty@tty1` and restart the service.

## Configure
Point the admin app at `‹pi-ip›:8770` (or `‹host›.local:8770`) and assign views
exactly as before — the panel picker, fit, target FPS, and click delay all work.

## Status / notes
- Renders **one screen** (the first configured). Multi-output KMSDRM is a future
  addition; the config/admin still manage multiple screens.
- Local views use a system font (DejaVu). If none is found, text views are blank
  but glassout still works — install `fonts-dejavu-core`.
- Frames are decoded on the CPU with `zune-jpeg` (pure Rust). Plenty fast on a
  Pi 4; if a Pi ever needs more, swapping in `turbojpeg` (NEON) is a drop-in.
- Config lives at `~/.config/learsim-glass/config.json`.
