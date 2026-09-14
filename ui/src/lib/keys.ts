import { summon } from "./actions.ts";

const EDITABLE = new Set(["INPUT", "TEXTAREA"]);
const APPLE = /Macintosh|iPhone|iPad/;

export function apple(agent: string): boolean {
  return APPLE.test(agent);
}

export function summoned(event: KeyboardEvent, mac: boolean): boolean {
  const key = event.code === "KeyK" || event.key.toLowerCase() === "k";
  const held = mac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  return key && held && !event.altKey && !event.shiftKey;
}

function editing(node: EventTarget | null): node is HTMLElement {
  return node instanceof HTMLElement && EDITABLE.has(node.tagName);
}

function focusFilter(doc: Document, event: KeyboardEvent): void {
  const field = doc.querySelector("[data-filter]");
  if (!(field instanceof HTMLElement)) return;
  event.preventDefault();
  field.focus();
}

function collapse(doc: Document): boolean {
  const open = doc.querySelectorAll("details[open]");
  for (const details of open) details.removeAttribute("open");
  return open.length > 0;
}

function up(doc: Document, event: KeyboardEvent): void {
  if (editing(event.target)) {
    event.target.blur();
    return;
  }
  if (collapse(doc)) return;
  const back = doc.querySelector("[data-up]") ?? doc.querySelector("[data-back]");
  if (back instanceof HTMLElement) back.click();
}

export function shortcut(doc: Document, mac: boolean): void {
  const [label, keys] = mac ? ["⌘K", "Meta+K"] : ["Ctrl+K", "Control+K"];
  for (const hint of doc.querySelectorAll("[data-chord]")) hint.textContent = label;
  for (const button of doc.querySelectorAll("[data-summon]")) button.setAttribute("aria-keyshortcuts", keys);
}

function summoner(event: Event): boolean {
  return event.target instanceof Element && event.target.closest("[data-summon]") !== null;
}

export function bind(doc: Document): void {
  const mac = apple(doc.defaultView?.navigator.userAgent ?? "");
  shortcut(doc, mac);
  doc.addEventListener("mousedown", (event) => {
    if (summoner(event)) event.preventDefault();
  });
  doc.addEventListener("click", (event) => {
    if (summoner(event)) summon();
  });
  doc.addEventListener("keydown", (event) => {
    if (summoned(event, mac)) {
      event.preventDefault();
      summon();
    }
    if (event.key === "/" && !editing(event.target)) focusFilter(doc, event);
    if (event.key === "Escape") up(doc, event);
  });
}
