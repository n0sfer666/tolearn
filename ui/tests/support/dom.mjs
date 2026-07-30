import { Window } from "happy-dom";

export function browser(url = "https://tolearn.local/") {
  const window = new Window({ url });
  globalThis.window = window;
  globalThis.document = window.document;
  globalThis.Node = window.Node;
  globalThis.Element = window.Element;
  globalThis.HTMLElement = window.HTMLElement;
  globalThis.Event = window.Event;
  globalThis.CustomEvent = window.CustomEvent;
  globalThis.localStorage = window.localStorage;
  return window;
}

export const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

export function toasts(window) {
  const said = [];
  window.addEventListener("tolearn:toast", (event) => said.push(event.detail));
  return said;
}
