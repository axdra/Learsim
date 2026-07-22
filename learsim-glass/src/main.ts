// Entry point for a glass screen window.
//
// Each kiosk window is labelled with its screen id. We read that label, ask the
// backend for the screen's current view + settings, render the matching local
// view, and re-render whenever the backend emits `screen-changed` for us.
//
// glassout screens never reach this code — the backend navigates those windows
// straight to the engine's viewer URL.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getRenderer, type ScreenState } from "./views/registry.ts";
import { escapeHtml } from "./views/util.ts";
import "./styles.css";

const container = document.getElementById("view");
let cleanup: (() => void) | null = null;

function render(screen: ScreenState) {
  if (!container) return;
  cleanup?.();
  const renderer = getRenderer(screen.viewId);
  cleanup = renderer(container, screen);
}

// Renders a readable error onto the screen using INLINE styles, so it is
// visible even if the stylesheet never loaded (the usual cause of a blank
// white kiosk screen). Far more useful on a headless panel than a blank page.
function renderError(message: string) {
  cleanup?.();
  cleanup = null;
  const el = container ?? document.body;
  el.innerHTML =
    '<div style="min-height:100vh;display:flex;align-items:center;justify-content:center;' +
    'background:#05070a;color:#ff6b6b;font-family:system-ui,sans-serif;padding:2rem;text-align:center">' +
    '<div><div style="color:#5c6b7a;font-size:.75rem;letter-spacing:.35em;text-transform:uppercase;' +
    'margin-bottom:1rem">learsim · glass</div>' +
    `<div style="font-size:1.05rem;max-width:64ch;white-space:pre-wrap">${escapeHtml(message)}</div></div></div>`;
}

// Surface uncaught errors on-screen instead of failing silently to white.
window.addEventListener("error", (e) => {
  renderError(`Script error: ${e.message}\n${e.filename ?? ""}:${e.lineno ?? ""}`);
});
window.addEventListener("unhandledrejection", (e) => {
  renderError(`Unhandled rejection: ${String(e.reason)}`);
});

async function boot() {
  let screenId = "?";
  try {
    screenId = getCurrentWindow().label;
    const state = await invoke<ScreenState>("get_screen_state", { screenId });
    render(state);

    // Live updates pushed by the control server when the admin reassigns a view.
    await listen<ScreenState>("screen-changed", (event) => {
      if (event.payload.id === screenId) {
        render(event.payload);
      }
    });
  } catch (err) {
    renderError(`Could not load screen "${screenId}": ${String(err)}`);
  }
}

boot();
