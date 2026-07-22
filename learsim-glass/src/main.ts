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
import "./styles.css";

const container = document.getElementById("view") as HTMLElement;
let cleanup: (() => void) | null = null;

function render(screen: ScreenState) {
  cleanup?.();
  const renderer = getRenderer(screen.viewId);
  cleanup = renderer(container, screen);
}

function renderError(message: string) {
  cleanup?.();
  cleanup = null;
  container.innerHTML = `<div class="fatal"><div class="fatal__title">learsim · glass</div><div class="fatal__msg">${message}</div></div>`;
}

async function boot() {
  const screenId = getCurrentWindow().label;

  try {
    const state = await invoke<ScreenState>("get_screen_state", { screenId });
    render(state);
  } catch (err) {
    renderError(`Could not load screen "${screenId}": ${String(err)}`);
  }

  // Live updates pushed by the control server when the admin reassigns a view.
  await listen<ScreenState>("screen-changed", (event) => {
    if (event.payload.id === screenId) {
      render(event.payload);
    }
  });
}

boot();
