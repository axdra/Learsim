// Frontend view registry.
//
// This registry holds the *local* views the display renders itself (standby,
// clock). glassout views are NOT rendered here — a glassout screen navigates
// the whole window to the engine's viewer URL (handled in the Rust backend), so
// by the time a glassout screen is showing, this SPA is no longer loaded.
//
// Adding a local view = write a `ViewRenderer` and register it below (and add a
// matching descriptor in the Rust `views.rs` manifest so the admin can pick it).

import { standbyView } from "./standby.ts";
import { clockView } from "./clock.ts";

export type ViewSettings = Record<string, unknown>;

export interface ScreenState {
  id: string;
  name: string;
  deviceName: string;
  viewId: string;
  settings: ViewSettings;
}

/**
 * A view renderer mounts UI into `container` and returns a cleanup function
 * that tears down timers/listeners. It is called again whenever the screen's
 * assignment or settings change.
 */
export type ViewRenderer = (container: HTMLElement, screen: ScreenState) => () => void;

const registry: Record<string, ViewRenderer> = {
  standby: standbyView,
  clock: clockView,
};

export function getRenderer(viewId: string): ViewRenderer {
  // Unknown / glassout / not-yet-implemented ids fall back to standby so a
  // screen is never blank.
  return registry[viewId] ?? standbyView;
}
