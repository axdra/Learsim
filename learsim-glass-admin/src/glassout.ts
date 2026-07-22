// glassout-client integration seam.
//
// The glassout SDK (`glassout-client`) isn't published to npm yet — it's shared
// privately by flyingart. When you drop it into this project (`npm add
// glassout-client` once it's live, or vendor the files), the helpers below
// light up automatically: panel discovery for the `panelId` dropdown and LAN
// engine discovery for the `engineUrl` field. Until then, everything falls back
// to manual entry and the Rust `/status` probe.
//
// The dynamic import is marked `@vite-ignore` and uses an assembled specifier
// so the bundler doesn't try to resolve a package that isn't installed; if the
// module is absent at runtime the import throws and we return null.

const SDK_SPECIFIER = ["glassout", "client"].join("-");

// Minimal shape we rely on — matches the documented SDK surface.
interface GlassOutClientLike {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  on(event: "panels", cb: (panels: Array<{ name: string }>) => void): void;
}
interface GlassOutSdk {
  GlassOutClient: new (opts: {
    name: string;
    appKey: string;
    mode: "client" | "host";
    host?: string;
    port?: number;
    connectionType?: "viewer" | "process";
  }) => GlassOutClientLike;
}

async function loadSdk(): Promise<GlassOutSdk | null> {
  try {
    // @vite-ignore
    return (await import(/* @vite-ignore */ SDK_SPECIFIER)) as unknown as GlassOutSdk;
  } catch {
    return null;
  }
}

/** True when the glassout-client SDK is available in this build. */
export async function sdkAvailable(): Promise<boolean> {
  return (await loadSdk()) !== null;
}

/** Parse an engine URL like "http://192.168.1.42:8787" into host + port. */
export function parseEngineUrl(engineUrl: string): { host: string; port: number } | null {
  try {
    const u = new URL(engineUrl);
    return { host: u.hostname, port: u.port ? Number(u.port) : 8787 };
  } catch {
    return null;
  }
}

/**
 * List the panel names a glassout engine is currently serving, via the SDK.
 * Returns null when the SDK isn't present (caller should fall back to manual
 * entry). Connects as a lightweight viewer client and resolves on the first
 * `panels` broadcast.
 */
export async function listPanels(engineUrl: string): Promise<string[] | null> {
  const sdk = await loadSdk();
  if (!sdk) return null;

  const target = parseEngineUrl(engineUrl);
  if (!target) return [];

  const engine = new sdk.GlassOutClient({
    name: "learsim-glass-admin",
    appKey: "learsim-glass-admin",
    mode: "client",
    host: target.host,
    port: target.port,
    connectionType: "viewer",
  });

  return new Promise<string[]>((resolve) => {
    const done = (panels: string[]) => {
      engine.disconnect().catch(() => {});
      resolve(panels);
    };
    const timeout = window.setTimeout(() => done([]), 4000);
    engine.on("panels", (panels) => {
      window.clearTimeout(timeout);
      done(panels.map((p) => p.name));
    });
    engine.connect().catch(() => {
      window.clearTimeout(timeout);
      done([]);
    });
  });
}
