import type { ScreenState, ViewRenderer } from "./registry.ts";

// Large clock view. Settings:
//   hourFormat  "24h" | "12h"   (default "24h")
//   showSeconds boolean         (default true)
//   showDate    boolean         (default true)
//   utc         boolean         (default false) — show UTC / Zulu time
export const clockView: ViewRenderer = (container: HTMLElement, screen: ScreenState) => {
  const hour12 = screen.settings.hourFormat === "12h";
  const showSeconds = screen.settings.showSeconds !== false;
  const showDate = screen.settings.showDate !== false;
  const utc = screen.settings.utc === true;
  const timeZone = utc ? "UTC" : undefined;

  container.innerHTML = `
    <div class="clock">
      <div class="clock__time" id="clock-time">--:--</div>
      <div class="clock__date" id="clock-date"></div>
      <div class="clock__id">${escapeHtml(screen.deviceName)} · ${escapeHtml(screen.name)}${
        utc ? " · ZULU" : ""
      }</div>
    </div>
  `;

  const timeEl = container.querySelector<HTMLElement>("#clock-time")!;
  const dateEl = container.querySelector<HTMLElement>("#clock-date")!;

  const timeFmt = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    ...(showSeconds ? { second: "2-digit" } : {}),
    hour12,
    timeZone,
  });
  const dateFmt = new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
    timeZone,
  });

  const tick = () => {
    const now = new Date();
    timeEl.textContent = utc ? `${timeFmt.format(now)}Z` : timeFmt.format(now);
    dateEl.textContent = showDate ? dateFmt.format(now) : "";
  };

  tick();
  // Re-render every 250ms so the seconds stay crisp without a per-second drift.
  const timer = window.setInterval(tick, showSeconds ? 250 : 1000);

  return () => {
    window.clearInterval(timer);
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
