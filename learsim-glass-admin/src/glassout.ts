// glassout engine discovery over plain HTTP.
//
// No SDK required: the engine exposes everything the admin needs on
// `GET /status` — the live panel list plus health/version/MSFS state. We reach
// it through the Rust `probe_engine` command (which proxies `<engineUrl>/status`
// and sidesteps CORS), so "List panels" and "Test engine" work against any
// reachable engine with zero extra dependencies.

import { probeEngine } from "./api.ts";

export interface EnginePanel {
  id: string;
  name: string;
  width?: number;
  height?: number;
}

export interface EngineStatus {
  ok?: boolean;
  version?: string;
  port?: number;
  uptime?: number;
  state?: {
    msfsConnection?: string;
    isAdmin?: boolean;
    [k: string]: unknown;
  };
  panels?: EnginePanel[];
  [k: string]: unknown;
}

/** Fetch and parse a glassout engine's `/status` snapshot. */
export async function fetchStatus(engineUrl: string): Promise<EngineStatus> {
  return (await probeEngine(engineUrl)) as EngineStatus;
}

/** The panels a glassout engine is currently serving, for the panelId picker. */
export async function listPanels(engineUrl: string): Promise<EnginePanel[]> {
  const status = await fetchStatus(engineUrl);
  return Array.isArray(status.panels) ? status.panels : [];
}

/**
 * Build the glassout viewer URL a glass screen will navigate to, mirroring the
 * display's Rust `build_view_url`. Used for "Copy viewer URL" so an operator
 * can open the exact same panel in a browser. Returns null if incomplete.
 */
export function buildViewerUrl(settings: Record<string, unknown>): string | null {
  const str = (k: string): string =>
    typeof settings[k] === "string" ? (settings[k] as string).trim() : "";
  const base = str("engineUrl").replace(/\/+$/, "");
  if (!base) return null;

  const path = str("path");
  if (path) return `${base}${path.startsWith("/") ? "" : "/"}${path}`;

  const panel = str("panelId");
  if (!panel) return null;

  const query: string[] = [];
  const fps = str("targetFps");
  if (fps && /^\d+$/.test(fps)) query.push(`fps=${fps}`);
  const fit = str("fit");
  if (fit && fit !== "contain") query.push(`fit=${fit}`);
  const clickDelay = str("clickDelay");
  if (clickDelay && /^\d+$/.test(clickDelay)) query.push(`clickDelay=${Math.min(5000, Number(clickDelay))}`);
  if (settings.debug === true) query.push("debug=1");

  const q = query.length ? `?${query.join("&")}` : "";
  return `${base}/panel/${encodeURIComponent(panel)}${q}`;
}

/** A short human-readable health line for the "Test engine" button. */
export function describeStatus(status: EngineStatus): string {
  const parts: string[] = [];
  if (status.version) parts.push(`v${status.version}`);
  const msfs = status.state?.msfsConnection;
  if (msfs) parts.push(`MSFS ${msfs}`);
  const count = status.panels?.length;
  if (typeof count === "number") parts.push(`${count} panel(s)`);
  return parts.length ? parts.join(" · ") : "reachable";
}
