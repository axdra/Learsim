import {
  addDevice,
  addScreen,
  deleteScreen,
  fetchDevice,
  listDevices,
  pingDevice,
  removeDevice,
  setScreen,
  DEFAULT_CONTROL_PORT,
  type DeviceSnapshot,
  type ScreenConfig,
  type StoredDevice,
  type ViewField,
} from "./api.ts";
import { buildViewerUrl, describeStatus, fetchStatus, listPanels } from "./glassout.ts";
import { clear, h } from "./dom.ts";
import "./styles.css";

interface Endpoint {
  host: string;
  port: number;
}

// --- app state --------------------------------------------------------------

let devices: StoredDevice[] = [];
let selected: Endpoint | null = null;
let snapshot: DeviceSnapshot | null = null;
// Liveness by "host:port" → online. Undefined = not yet checked.
const online = new Map<string, boolean>();

const root = document.getElementById("app") as HTMLElement;

const endpointKey = (e: Endpoint): string => `${e.host}:${e.port}`;

// Surface uncaught errors on-screen instead of failing silently to a blank
// window, and mirror them into the status bar when the shell is up.
function reportFatal(message: string) {
  const bar = document.getElementById("statusbar");
  if (bar) {
    bar.textContent = message;
    bar.dataset.kind = "error";
    return;
  }
  root.innerHTML =
    '<div style="padding:2rem;color:#ff6b6b;font-family:system-ui,sans-serif;white-space:pre-wrap">' +
    `${message.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[c] ?? c)}</div>`;
}
window.addEventListener("error", (e) =>
  reportFatal(`Script error: ${e.message}\n${e.filename ?? ""}:${e.lineno ?? ""}`),
);
window.addEventListener("unhandledrejection", (e) =>
  reportFatal(`Unhandled rejection: ${String(e.reason)}`),
);

function setStatus(msg: string, kind: "info" | "error" = "info") {
  const bar = document.getElementById("statusbar");
  if (bar) {
    bar.textContent = msg;
    bar.dataset.kind = kind;
  }
}

function sameEndpoint(a: Endpoint | null, b: Endpoint | null): boolean {
  return !!a && !!b && a.host === b.host && a.port === b.port;
}

// --- top-level render -------------------------------------------------------

function render() {
  clear(root);
  root.append(
    h("div", { class: "layout" }, renderSidebar(), renderMain()),
    h("div", { id: "statusbar", class: "statusbar", dataset: { kind: "info" } }, "Ready"),
  );
}

function renderSidebar(): HTMLElement {
  const list = h(
    "div",
    { class: "device-list" },
    ...devices.map((d) => {
      const active = sameEndpoint(selected, d);
      const status = online.get(endpointKey(d));
      const dotClass =
        status === undefined ? "unknown" : status ? "online" : "offline";
      return h(
        "button",
        {
          class: `device-item${active ? " device-item--active" : ""}`,
          onclick: () => selectDevice(d),
        },
        h("span", {
          class: `status-dot status-dot--${dotClass}`,
          dataset: { ep: endpointKey(d) },
          title:
            status === undefined ? "Checking…" : status ? "Online" : "Offline",
        }),
        h(
          "div",
          { class: "device-item__body" },
          h("span", { class: "device-item__name" }, d.label ?? `${d.host}`),
          h("span", { class: "device-item__addr" }, `${d.host}:${d.port}`),
        ),
        h("span", {
          class: "device-item__remove",
          title: "Forget device",
          onclick: (e: Event) => {
            e.stopPropagation();
            forgetDevice(d);
          },
        }, "×"),
      );
    }),
    devices.length === 0 && h("p", { class: "muted pad" }, "No devices yet."),
  );

  const hostInput = h("input", {
    class: "input",
    placeholder: "host or IP",
    id: "add-host",
  });
  const portInput = h("input", {
    class: "input input--port",
    type: "number",
    value: String(DEFAULT_CONTROL_PORT),
    id: "add-port",
  });

  const addForm = h(
    "form",
    {
      class: "add-form",
      onsubmit: (e: Event) => {
        e.preventDefault();
        const host = hostInput.value.trim();
        const port = Number(portInput.value) || DEFAULT_CONTROL_PORT;
        if (host) connectNewDevice(host, port);
      },
    },
    h("div", { class: "add-form__row" }, hostInput, portInput),
    h("button", { class: "btn btn--primary", type: "submit" }, "Add device"),
  );

  return h(
    "aside",
    { class: "sidebar" },
    h("div", { class: "brand" }, h("span", { class: "brand__dot" }), "learsim · glass admin"),
    h("h2", { class: "sidebar__title" }, "Devices"),
    list,
    addForm,
  );
}

function renderMain(): HTMLElement {
  if (!selected) {
    return h(
      "main",
      { class: "main main--empty" },
      h("div", { class: "empty" }, "Select or add a glass device to configure its screens."),
    );
  }
  if (!snapshot) {
    return h("main", { class: "main main--empty" }, h("div", { class: "empty" }, "Connecting…"));
  }

  const header = h(
    "header",
    { class: "device-header" },
    h(
      "div",
      {},
      h("h1", { class: "device-title" }, snapshot.device.name),
      h(
        "div",
        { class: "device-sub" },
        `${selected.host}:${selected.port} · ${snapshot.screens.length} screen(s) · control port ${snapshot.port}`,
      ),
    ),
    h(
      "div",
      { class: "device-actions" },
      h("button", { class: "btn", onclick: () => refresh() }, "Refresh"),
      h("button", { class: "btn btn--primary", onclick: () => onAddScreen() }, "＋ Add screen"),
    ),
  );

  const screens = h(
    "div",
    { class: "screens" },
    ...snapshot.screens.map((s) => renderScreenCard(snapshot!, s)),
  );

  return h("main", { class: "main" }, header, screens);
}

// --- screen card ------------------------------------------------------------

function renderScreenCard(snap: DeviceSnapshot, screen: ScreenConfig): HTMLElement {
  // Working copy so edits are batched until "Apply".
  const draft = {
    name: screen.name,
    viewId: screen.viewId,
    settings: { ...screen.settings } as Record<string, unknown>,
  };

  const settingsHost = h("div", { class: "settings" });

  const renderSettings = () => {
    clear(settingsHost);
    const view = snap.views.find((v) => v.id === draft.viewId);
    if (!view) return;
    settingsHost.append(h("p", { class: "view-desc" }, view.description));
    for (const field of view.fields) {
      settingsHost.append(renderField(field, draft.settings));
    }
    if (view.id === "glassout") {
      settingsHost.append(renderGlassoutTools(draft.settings));
    }
  };

  const nameInput = h("input", {
    class: "input",
    value: draft.name,
    oninput: (e: Event) => (draft.name = (e.target as HTMLInputElement).value),
  });

  const viewSelect = h(
    "select",
    {
      class: "input",
      onchange: (e: Event) => {
        draft.viewId = (e.target as HTMLSelectElement).value;
        // Seed defaults for the newly-selected view's fields.
        const view = snap.views.find((v) => v.id === draft.viewId);
        view?.fields.forEach((f) => {
          if (!(f.key in draft.settings)) draft.settings[f.key] = f.default;
        });
        renderSettings();
      },
    },
    ...snap.views.map((v) =>
      h("option", { value: v.id, selected: v.id === draft.viewId }, v.name),
    ),
  );

  renderSettings();

  return h(
    "section",
    { class: "card" },
    h(
      "div",
      { class: "card__top" },
      h("div", { class: "card__id" }, screen.id, screen.monitor != null
        ? h("span", { class: "chip" }, `monitor ${screen.monitor}`)
        : null),
      h(
        "button",
        {
          class: "btn btn--ghost btn--danger",
          onclick: () => onDeleteScreen(screen),
        },
        "Remove",
      ),
    ),
    h(
      "div",
      { class: "field-row" },
      h("label", { class: "field" }, h("span", { class: "field__label" }, "Name"), nameInput),
      h("label", { class: "field" }, h("span", { class: "field__label" }, "View"), viewSelect),
    ),
    settingsHost,
    h(
      "div",
      { class: "card__actions" },
      h(
        "button",
        { class: "btn btn--primary", onclick: () => onApply(screen.id, draft) },
        "Apply",
      ),
    ),
  );
}

function renderField(field: ViewField, settings: Record<string, unknown>): HTMLElement {
  const current = field.key in settings ? settings[field.key] : field.default;

  if (field.kind === "boolean") {
    const input = h("input", {
      type: "checkbox",
      checked: current === true,
      onchange: (e: Event) => (settings[field.key] = (e.target as HTMLInputElement).checked),
    });
    return h("label", { class: "field field--check" }, input, h("span", {}, field.label));
  }

  if (field.kind === "select") {
    const select = h(
      "select",
      {
        class: "input",
        onchange: (e: Event) => (settings[field.key] = (e.target as HTMLSelectElement).value),
      },
      ...(field.options ?? []).map((o) =>
        h("option", { value: o.value, selected: o.value === current }, o.label),
      ),
    );
    return h("label", { class: "field" }, h("span", { class: "field__label" }, field.label), select);
  }

  // text
  const datalistId = `dl-${field.key}-${Math.random().toString(36).slice(2, 7)}`;
  const input = h("input", {
    class: "input",
    value: current == null ? "" : String(current),
    list: field.key === "panelId" ? datalistId : undefined,
    oninput: (e: Event) => (settings[field.key] = (e.target as HTMLInputElement).value),
  });
  const wrap = h("label", { class: "field" }, h("span", { class: "field__label" }, field.label), input);
  if (field.key === "panelId") {
    wrap.append(h("datalist", { id: datalistId }));
  }
  return wrap;
}

// glassout helpers: check the engine's /status and populate the panelId
// picker from its live panel list — all over plain HTTP, no SDK needed.
function renderGlassoutTools(settings: Record<string, unknown>): HTMLElement {
  const result = h("span", { class: "muted" }, "");

  // Find the panelId <datalist> belonging to the same screen card.
  const cardDatalist = (fromButton: HTMLElement): HTMLDataListElement | null =>
    fromButton.closest(".card")?.querySelector<HTMLDataListElement>("datalist") ?? null;

  const test = h(
    "button",
    {
      class: "btn btn--ghost",
      type: "button",
      onclick: async () => {
        const engineUrl = String(settings.engineUrl ?? "");
        if (!engineUrl) return void (result.textContent = "Set an engine URL first.");
        result.textContent = "Probing…";
        try {
          result.textContent = `Engine ${describeStatus(await fetchStatus(engineUrl))}`;
        } catch (err) {
          result.textContent = `Unreachable: ${String(err)}`;
        }
      },
    },
    "Test engine",
  );

  const discover = h(
    "button",
    {
      class: "btn btn--ghost",
      type: "button",
      onclick: async (e: Event) => {
        // Capture the button now: after the await, e.currentTarget is null.
        const button = e.currentTarget as HTMLElement;
        const engineUrl = String(settings.engineUrl ?? "");
        if (!engineUrl) return void (result.textContent = "Set an engine URL first.");
        result.textContent = "Listing panels…";
        try {
          const panels = await listPanels(engineUrl);
          const dl = cardDatalist(button);
          if (dl) {
            clear(dl);
            panels.forEach((p) =>
              dl.append(h("option", { value: p.id }, p.name || p.id)),
            );
          }
          result.textContent = panels.length
            ? `Found ${panels.length} panel(s) — pick one in the Panel id field.`
            : "Engine reachable but reported no panels.";
        } catch (err) {
          result.textContent = `Could not list panels: ${String(err)}`;
        }
      },
    },
    "List panels",
  );

  const copy = h(
    "button",
    {
      class: "btn btn--ghost",
      type: "button",
      onclick: async () => {
        const url = buildViewerUrl(settings);
        if (!url) return void (result.textContent = "Set engine URL and panel id first.");
        try {
          await navigator.clipboard.writeText(url);
          result.textContent = `Copied: ${url}`;
        } catch {
          result.textContent = url;
        }
      },
    },
    "Copy viewer URL",
  );

  return h("div", { class: "glassout-tools" }, test, discover, copy, result);
}

// --- actions ----------------------------------------------------------------

async function connectNewDevice(host: string, port: number) {
  setStatus(`Connecting to ${host}:${port}…`);
  try {
    snapshot = await addDevice(host, port);
    selected = { host, port };
    devices = await listDevices();
    setStatus(`Connected to ${snapshot.device.name}.`);
    render();
  } catch (err) {
    setStatus(`Could not add ${host}:${port}: ${String(err)}`, "error");
  }
}

async function selectDevice(d: StoredDevice) {
  selected = { host: d.host, port: d.port };
  snapshot = null;
  render();
  await refresh();
}

async function refresh() {
  if (!selected) return;
  setStatus("Refreshing…");
  try {
    snapshot = await fetchDevice(selected.host, selected.port);
    setStatus(`Updated ${snapshot.device.name}.`);
    render();
  } catch (err) {
    setStatus(`Refresh failed: ${String(err)}`, "error");
  }
}

async function forgetDevice(d: StoredDevice) {
  await removeDevice(d.host, d.port);
  if (sameEndpoint(selected, d)) {
    selected = null;
    snapshot = null;
  }
  devices = await listDevices();
  setStatus(`Forgot ${d.host}:${d.port}.`);
  render();
}

async function onApply(
  screenId: string,
  draft: { name: string; viewId: string; settings: Record<string, unknown> },
) {
  if (!selected || !snapshot) return;
  setStatus(`Applying to ${screenId}…`);
  try {
    const updated = await setScreen(selected.host, selected.port, screenId, {
      name: draft.name,
      viewId: draft.viewId,
      settings: draft.settings,
    });
    const idx = snapshot.screens.findIndex((s) => s.id === screenId);
    if (idx >= 0) snapshot.screens[idx] = updated;
    setStatus(`${screenId} now showing "${updated.viewId}".`);
    render();
  } catch (err) {
    setStatus(`Apply failed: ${String(err)}`, "error");
  }
}

async function onAddScreen() {
  if (!selected) return;
  try {
    await addScreen(selected.host, selected.port, {});
    await refresh();
  } catch (err) {
    setStatus(`Add screen failed: ${String(err)}`, "error");
  }
}

async function onDeleteScreen(screen: ScreenConfig) {
  if (!selected) return;
  try {
    await deleteScreen(selected.host, selected.port, screen.id);
    await refresh();
  } catch (err) {
    setStatus(`Remove failed: ${String(err)}`, "error");
  }
}

// --- liveness ---------------------------------------------------------------

// Ping every known device's /api/health and refresh the sidebar dots. Only the
// sidebar re-renders, so it never clobbers unsaved edits in a screen card.
async function pingAll() {
  await Promise.all(
    devices.map(async (d) => {
      try {
        online.set(endpointKey(d), await pingDevice(d.host, d.port));
      } catch {
        online.set(endpointKey(d), false);
      }
    }),
  );
  refreshDots();
}

// Update the sidebar status dots in place — never rebuilds the sidebar, so a
// half-typed entry in the add-device form is left untouched.
function refreshDots() {
  root.querySelectorAll<HTMLElement>(".status-dot[data-ep]").forEach((dot) => {
    const key = dot.dataset.ep ?? "";
    const status = online.get(key);
    const cls = status === undefined ? "unknown" : status ? "online" : "offline";
    dot.className = `status-dot status-dot--${cls}`;
    dot.title = status === undefined ? "Checking…" : status ? "Online" : "Offline";
  });
}

// --- boot -------------------------------------------------------------------

async function boot() {
  render();
  try {
    devices = await listDevices();
    render();
    pingAll();
    window.setInterval(pingAll, 15000);
  } catch (err) {
    setStatus(`Could not load devices: ${String(err)}`, "error");
  }
}

boot();
