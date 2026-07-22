import type { ScreenState, ViewRenderer } from "./registry.ts";
import { escapeHtml } from "./util.ts";

// Idle screen: shows the device + screen name so an operator can identify a
// physical panel at a glance. An optional `message` setting overrides the hint.
export const standbyView: ViewRenderer = (container: HTMLElement, screen: ScreenState) => {
  const message =
    typeof screen.settings.message === "string" && screen.settings.message.trim()
      ? screen.settings.message.trim()
      : "Standby";

  container.innerHTML = `
    <div class="flex flex-col gap-5 text-center">
      <div class="text-xs uppercase tracking-[0.5em] text-muted">learsim · glass</div>
      <div class="text-[clamp(2rem,8vw,6rem)] font-thin tracking-[0.04em]">${escapeHtml(message)}</div>
      <div class="text-base tracking-[0.08em] text-muted">
        <span>${escapeHtml(screen.deviceName)}</span>
        <span class="mx-2 text-accent">•</span>
        <span>${escapeHtml(screen.name)}</span>
      </div>
    </div>
  `;

  return () => {
    container.innerHTML = "";
  };
};
