# Collection of Learsim code

## Glass cockpit displays

- **[`learsim-glass`](learsim-glass/)** — kiosk display app that hosts
  [glassout](https://glassout.flyingart.dev) views (and local standby/clock
  views) fullscreen. Runs headless on a Raspberry Pi and on Windows/macOS. One
  screen config per monitor, so two monitors on one Pi get two independent
  configs.
- **[`learsim-glass-admin`](learsim-glass-admin/)** — desktop app to configure,
  over the LAN, which view each glass screen shows.
- **[`learsim-glass-native`](learsim-glass-native/)** — a native (no-webview)
  display client that renders glassout panels directly via SDL2/KMSDRM. Much
  lighter on a Raspberry Pi than the webview build (no browser engine, no
  compositor) and speaks the same control API, so the admin manages it the same.

See each app's README for architecture, build, and headless-Pi setup.

## Other
