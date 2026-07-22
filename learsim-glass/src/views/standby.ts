import type { ScreenState, ViewRenderer } from "./registry.ts";

// Idle screen: shows the device + screen name so an operator can identify a
// physical panel at a glance. An optional `message` setting overrides the hint.
export const standbyView: ViewRenderer = (container: HTMLElement, screen: ScreenState) => {
  const message =
    typeof screen.settings.message === "string" && screen.settings.message.trim()
      ? screen.settings.message.trim()
      : "Standby";

  container.innerHTML = `
    <div class="standby">
      <div class="standby__mark">learsim · glass</div>
      <div class="standby__message">${escapeHtml(message)}</div>
      <div class="standby__id">
        <span>${escapeHtml(screen.deviceName)}</span>
        <span class="standby__dot">•</span>
        <span>${escapeHtml(screen.name)}</span>
      </div>
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
