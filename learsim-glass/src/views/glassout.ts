import type { ScreenState, ViewRenderer } from "./registry.ts";
import { escapeHtml } from "./util.ts";

// Placeholder shown when a screen is assigned the glassout view but its window
// is parked on our app document — which the backend does when the engine is
// unreachable or not yet configured. The Rust monitor loop navigates the window
// straight to the engine's panel viewer as soon as `/status` responds, so this
// screen self-recovers without any action here.
export const glassoutView: ViewRenderer = (container: HTMLElement, screen: ScreenState) => {
  const engineUrl =
    typeof screen.settings.engineUrl === "string" ? screen.settings.engineUrl.trim() : "";
  const panelId =
    typeof screen.settings.panelId === "string" ? screen.settings.panelId.trim() : "";

  const configured = engineUrl.length > 0;

  container.innerHTML = `
    <div class="flex flex-col items-center gap-4 text-center">
      <div class="mb-1 h-12 w-12 animate-spin rounded-full border-[3px] border-muted border-t-accent" aria-hidden="true"></div>
      <div class="text-[clamp(1.5rem,5vw,3rem)] font-thin tracking-[0.04em]">${
        configured ? "Connecting to glassout" : "Glassout not configured"
      }</div>
      <div class="font-mono text-base opacity-75">
        ${configured ? escapeHtml(engineUrl) : "Set an engine URL in the admin app."}
        ${panelId ? `<span class="mx-2 text-accent">•</span>${escapeHtml(panelId)}` : ""}
      </div>
      <div class="mt-2 text-xs uppercase tracking-[0.14em] text-muted">${escapeHtml(
        screen.deviceName,
      )} · ${escapeHtml(screen.name)}</div>
    </div>
  `;

  return () => {
    container.innerHTML = "";
  };
};
