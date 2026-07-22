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
