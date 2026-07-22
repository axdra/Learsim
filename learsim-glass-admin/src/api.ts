// Typed wrappers over the Rust command layer (see src-tauri/src/client.rs).

import { invoke } from "@tauri-apps/api/core";

export interface DeviceInfo {
  id: string;
  name: string;
}

export interface ViewOption {
  value: string;
  label: string;
}

export interface ViewField {
  key: string;
  label: string;
  kind: "text" | "boolean" | "select";
  options?: ViewOption[];
  default: unknown;
}

export interface ViewDescriptor {
  id: string;
  name: string;
  description: string;
  fields: ViewField[];
}

export interface ScreenConfig {
  id: string;
  name: string;
  monitor: number | null;
  viewId: string;
  settings: Record<string, unknown>;
}

export interface DeviceSnapshot {
  device: DeviceInfo;
  port: number;
  screens: ScreenConfig[];
  views: ViewDescriptor[];
}

export interface StoredDevice {
  host: string;
  port: number;
  label: string | null;
}

export const DEFAULT_CONTROL_PORT = 8770;

export function listDevices(): Promise<StoredDevice[]> {
  return invoke<StoredDevice[]>("list_devices");
}

export function addDevice(host: string, port: number): Promise<DeviceSnapshot> {
  return invoke<DeviceSnapshot>("add_device", { host, port });
}

export function removeDevice(host: string, port: number): Promise<void> {
  return invoke("remove_device", { host, port });
}

export function fetchDevice(host: string, port: number): Promise<DeviceSnapshot> {
  return invoke<DeviceSnapshot>("fetch_device", { host, port });
}

export interface SetScreenBody {
  viewId?: string;
  settings?: Record<string, unknown>;
  name?: string;
}

export function setScreen(
  host: string,
  port: number,
  screenId: string,
  body: SetScreenBody,
): Promise<ScreenConfig> {
  return invoke<ScreenConfig>("set_screen", { host, port, screenId, body });
}

export function addScreen(
  host: string,
  port: number,
  body: { name?: string; monitor?: number },
): Promise<ScreenConfig> {
  return invoke<ScreenConfig>("add_screen", { host, port, body });
}

export function deleteScreen(host: string, port: number, screenId: string): Promise<void> {
  return invoke("delete_screen", { host, port, screenId });
}

/** Reachability check for a glassout engine URL (GET {engineUrl}/status). */
export function probeEngine(engineUrl: string): Promise<unknown> {
  return invoke("probe_engine", { engineUrl });
}
