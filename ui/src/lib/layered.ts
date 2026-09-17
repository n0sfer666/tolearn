import { pressed } from "./shortcuts";

export interface Layer {
  keyed: (event: KeyboardEvent) => void;
  held: (event: FocusEvent) => void;
  aside: (event: MouseEvent) => void;
}

const STOPS = "button";

export function layered(
  box: () => HTMLElement | undefined,
  close: () => void,
  picks: string = STOPS,
): Layer {
  const stops = (): HTMLElement[] => [...(box()?.querySelectorAll<HTMLElement>(picks) ?? [])];

  const keyed = (event: KeyboardEvent) => {
    event.stopPropagation();
    if (pressed(event, "close")) {
      event.preventDefault();
      close();
      return;
    }
    if (!pressed(event, "cycle")) return;
    const row = stops();
    if (row.length === 0) return;
    const at = row.findIndex((stop) => stop === document.activeElement);
    event.preventDefault();
    row[(at + (event.shiftKey ? -1 : 1) + row.length) % row.length]?.focus();
  };

  const held = (event: FocusEvent) => {
    const next = event.relatedTarget;
    if (next instanceof Node && box()?.contains(next) === true) return;
    queueMicrotask(() => stops()[0]?.focus());
  };

  const aside = (event: MouseEvent) => {
    if (event.target !== event.currentTarget) return;
    close();
  };

  return { keyed, held, aside };
}
