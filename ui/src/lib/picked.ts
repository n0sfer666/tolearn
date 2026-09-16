export interface Aim {
  block: string;
  fragment: string;
}

export interface Spot {
  top: number;
  left: number;
  below: boolean;
}

export interface Picked {
  aim: Aim;
  range: Range;
}

const BLOCK = "[data-block]:not([data-block='heading'])";
const ROOM = 64;
const EDGE = 48;

export function blockOf(node: Node | null, within: Element): HTMLElement | null {
  const start = node instanceof Element ? node : (node?.parentElement ?? null);
  const block = start?.closest(BLOCK) ?? null;
  if (!(block instanceof HTMLElement) || block.id === "" || !within.contains(block)) return null;
  return block;
}

export function picked(doc: Document, within: Element): Picked | null {
  const selection = doc.defaultView?.getSelection() ?? null;
  if (selection === null || selection.isCollapsed || selection.rangeCount === 0) return null;
  const range = selection.getRangeAt(0);
  const block = blockOf(range.commonAncestorContainer, within);
  if (block === null) return null;
  const fragment = selection.toString().trim();
  return fragment === "" ? null : { aim: { block: block.id, fragment }, range };
}

export function focused(doc: Document, within: Element): HTMLElement | null {
  return blockOf(doc.activeElement, within);
}

export function spotted(rect: DOMRect, width: number): Spot {
  const below = rect.top < ROOM;
  const middle = rect.left + rect.width / 2;
  const left = Math.min(Math.max(middle, EDGE), Math.max(width - EDGE, EDGE));
  return { top: below ? rect.bottom : rect.top, left, below };
}
