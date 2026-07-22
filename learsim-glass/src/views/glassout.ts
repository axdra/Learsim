import type { ScreenState, ViewRenderer } from "./registry.ts";

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
    <div class="glassout">
      <div class="glassout__spinner" aria-hidden="true"></div>
      <div class="glassout__title">${configured ? "Connecting to glassout" : "Glassout not configured"}</div>
      <div class="glassout__detail">
        ${configured ? escapeHtml(engineUrl) : "Set an engine URL in the admin app."}
        ${panelId ? `<span class="glassout__dot">•</span>${escapeHtml(panelId)}` : ""}
      </div>
      <div class="glassout__id">${escapeHtml(screen.deviceName)} · ${escapeHtml(screen.name)}</div>
    </div>
  `;

  return () => {
    container.innerHTML = "";
  };
};

function escapeHtml(input: string): string {
  return input.replace(
    /[&<>"']/g,
    (c) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[c] ?? c,
  );
}
