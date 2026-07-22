import type { ScreenState, ViewRenderer } from "./registry.ts";
import { escapeHtml } from "./util.ts";

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
    <div class="flex flex-col gap-2 text-center">
      <div id="clock-time" class="text-[clamp(4rem,20vw,16rem)] font-thin leading-none tracking-tight tabular-nums">--:--</div>
      <div id="clock-date" class="text-[clamp(1rem,3vw,2rem)] tracking-[0.1em] text-muted"></div>
      <div class="mt-6 text-sm uppercase tracking-[0.14em] text-muted">${escapeHtml(
        screen.deviceName,
      )} · ${escapeHtml(screen.name)}${utc ? " · ZULU" : ""}</div>
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
