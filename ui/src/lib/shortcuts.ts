export type Scope = "screen" | "layer";

export interface Shortcut {
  readonly id: string;
  readonly scope: Scope;
  readonly key: string;
  readonly label: string;
  readonly chord?: boolean;
}

export const SHORTCUTS = [
  { id: "summon", scope: "screen", key: "k", label: "⌘K", chord: true },
  { id: "filter", scope: "screen", key: "/", label: "/" },
  { id: "up", scope: "screen", key: "Escape", label: "Esc" },
  { id: "previous", scope: "layer", key: "ArrowUp", label: "↑" },
  { id: "next", scope: "layer", key: "ArrowDown", label: "↓" },
  { id: "run", scope: "layer", key: "Enter", label: "Enter" },
  { id: "close", scope: "layer", key: "Escape", label: "Esc" },
  { id: "cycle", scope: "layer", key: "Tab", label: "Tab" },
] as const satisfies readonly Shortcut[];

export type ShortcutId = (typeof SHORTCUTS)[number]["id"];

export const SCOPES: readonly Scope[] = ["screen", "layer"];

const LETTER = /^[a-z]$/;

export function chorded(shortcut: Shortcut): boolean {
  return shortcut.chord === true;
}

export function pressed(event: KeyboardEvent, id: ShortcutId): boolean {
  const key = SHORTCUTS.find((shortcut) => shortcut.id === id)?.key;
  if (key === undefined) return false;
  if (!LETTER.test(key)) return event.key === key;
  return event.code === `Key${key.toUpperCase()}` || event.key.toLowerCase() === key;
}
