// Tiny DOM helper — keeps the UI code declarative without a framework.

type Attrs = Record<string, unknown>;
type Child = Node | string | null | undefined | false;

// Keys that must be set as HTML attributes rather than DOM properties. `list`
// in particular is a read-only property on <input> (it returns the resolved
// <datalist> element), so assigning to it throws — it has to go through
// setAttribute to associate the datalist by id.
const ATTR_ONLY = new Set(["list"]);

export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Attrs = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null || value === false) continue;
    if (key === "class") {
      el.className = String(value);
    } else if (key === "dataset" && typeof value === "object") {
      Object.assign(el.dataset, value as Record<string, string>);
    } else if (key.startsWith("on") && typeof value === "function") {
      el.addEventListener(key.slice(2).toLowerCase(), value as EventListener);
    } else if (ATTR_ONLY.has(key)) {
      el.setAttribute(key, String(value));
    } else if (key in el) {
      // Property assignment (value, checked, disabled, textContent, …).
      (el as unknown as Record<string, unknown>)[key] = value;
    } else {
      el.setAttribute(key, String(value));
    }
  }
  for (const child of children) {
    if (child == null || child === false) continue;
    el.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return el;
}

export function clear(el: HTMLElement): void {
  el.replaceChildren();
}
