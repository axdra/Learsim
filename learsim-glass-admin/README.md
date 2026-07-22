# learsim-glass-admin

Desktop **control** app for [`learsim-glass`](../learsim-glass) displays.
Connect to a glass device over the LAN and choose which view each of its screens
shows.

![role](https://img.shields.io/badge/runs%20on-Windows%20%7C%20macOS%20%7C%20Linux-informational)

---

## What it does

- **Add a device** by `host:port` (the glass control server, default `8770`).
  The device is validated (`GET /api/device`) and remembered.
- For each of the device's **screens**, pick a **view** and edit its settings.
  The form is built dynamically from the view manifest the device reports, so it
  always matches what that device supports.
- **Add / remove screens** and **rename** them.
- Changes are pushed to the device and apply live.

All HTTP to devices goes through the Rust backend (`src-tauri/src/client.rs`),
so there are no CORS or mixed-content issues.

## glassout views

When a screen's view is **glassout**, you set the engine URL and panel:

- **Test engine** — probes `GET <engineUrl>/status` to confirm the engine is
  reachable.
- **List panels** — enumerates the engine's panels into the `panelId`
  autocomplete **when the `glassout-client` SDK is installed** (see below).
  Without it, type the panel id manually.

### Enabling panel / engine discovery (glassout-client SDK)

The [`glassout-client`](https://glassout.flyingart.dev/library/developers/architecture)
TypeScript SDK isn't on npm yet — request the files from flyingart, then:

```bash
npm add glassout-client       # once it's published, or vendor the files
```

The integration seam (`src/glassout.ts`) loads it lazily, so **List panels**
and (future) LAN engine discovery light up automatically once it resolves. No
other code changes needed.

## Develop

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

(Installer bundling is off by default; see the note in the glass app README.)
