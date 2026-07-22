// Frontend view registry.
//
// Holds the views the SPA renders itself: local views (standby, clock, test
// pattern) plus the glassout *placeholder*. A glassout screen normally shows
// the engine's viewer URL directly (the SPA isn't loaded then); the SPA only
// renders the glassout entry as a "connecting…" placeholder while the backend
// has the window parked on our app document (engine down / not configured).
//
// Adding a local view = write a `ViewRenderer` and register it below (and add a
// matching descriptor in the Rust `views.rs` manifest so the admin can pick it).

import { standbyView } from "./standby.ts";
import { clockView } from "./clock.ts";
import { glassoutView } from "./glassout.ts";
import { testPatternView } from "./testpattern.ts";

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
  glassout: glassoutView,
  testpattern: testPatternView,
};

export function getRenderer(viewId: string): ViewRenderer {
  // Unknown / not-yet-implemented ids fall back to standby so a screen is
  // never blank.
  return registry[viewId] ?? standbyView;
}
