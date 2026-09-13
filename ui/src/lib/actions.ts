export const SUMMONED = "tolearn:actions";

export interface Offer {
  label: string;
  element: HTMLElement;
}

const SOURCES = ["[data-action]", 'body > header a[href]:not([aria-current="page"])'];
const UNAVAILABLE = '[disabled], [aria-disabled="true"]';
const CONCEALED = "[hidden], [inert]";

export function summon(): void {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent(SUMMONED));
}

export function offered(doc: Document): Offer[] {
  const seen = new Set<Element>();
  const found: Offer[] = [];

  for (const source of SOURCES) {
    for (const element of doc.querySelectorAll(source)) {
      if (seen.has(element) || !available(element)) continue;
      seen.add(element);
      const label = labelled(element);
      if (label !== "") found.push({ label, element });
    }
  }

  return found;
}

export function narrowed(offers: readonly Offer[], query: string): Offer[] {
  const wanted = query.trim().toLocaleLowerCase();
  return offers.filter((offer) => offer.label.toLocaleLowerCase().includes(wanted));
}

function available(element: Element): element is HTMLElement {
  return element instanceof HTMLElement && !element.matches(UNAVAILABLE) && element.closest(CONCEALED) === null;
}

function labelled(element: HTMLElement): string {
  return (element.getAttribute("aria-label") ?? element.textContent ?? "").replace(/\s+/g, " ").trim();
}
