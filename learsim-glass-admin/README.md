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
- **Live status dots** in the sidebar — each device is pinged (`/api/health`)
  every 15s and shown green (online) / red (offline). Pinging only updates the
  dots, so it never disturbs edits in progress.

All HTTP to devices goes through the Rust backend (`src-tauri/src/client.rs`),
so there are no CORS or mixed-content issues.

## glassout views

When a screen's view is **glassout**, you set the engine URL and panel:

- **Test engine** — reads `GET <engineUrl>/status` and reports version, MSFS
  connection state, and panel count.
- **List panels** — reads the same `/status` and fills the `panelId`
  autocomplete with the engine's live panel list.
- **Copy viewer URL** — copies the exact `/panel/{id}?…` URL the screen will
  show, so you can open it in a browser to verify.

### No SDK required

Everything above uses the engine's plain **HTTP `/status`** endpoint (which
returns `panels: [{ id, name, width, height }, …]`), proxied through the Rust
backend to avoid CORS. The private `glassout-client` TypeScript SDK is **not**
needed — discovery and the viewer URLs are all reachable over HTTP. `/status`
handling lives in `src/glassout.ts`.

## Develop

```bash
npm install
npm run tauri dev
```

Frontend stack: **TypeScript + Vite + Tailwind CSS v4** (tokens + component
classes in `src/styles.css`). Tooling: **ESLint** (`npm run lint`) and
**Prettier** with Tailwind class sorting (`npm run format`).

## Build

```bash
npm run tauri build
```

(Installer bundling is off by default; see the note in the glass app README.)
