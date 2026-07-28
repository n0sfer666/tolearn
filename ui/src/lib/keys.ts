const EDITABLE = new Set(["INPUT", "TEXTAREA"]);

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
  const back = doc.querySelector("[data-back]");
  if (back instanceof HTMLElement) back.click();
}

export function bind(doc: Document): void {
  doc.addEventListener("keydown", (event) => {
    if (event.key === "/" && !editing(event.target)) focusFilter(doc, event);
    if (event.key === "Escape") up(doc, event);
  });
}
