export const NAMED = "tolearn:name";
export const NAMES = "tolearn.names";

export interface Name {
  id: string;
  title: string;
}

export type Kind = "program" | "topic";

export function name(kind: Kind, id: string, title: string): void {
  if (id === "" || title === "" || typeof window === "undefined") return;
  keep(kind, { id, title });
  window.dispatchEvent(new CustomEvent(NAMED, { detail: { kind, id, title } }));
}

function keep(kind: Kind, named: Name): void {
  if (typeof localStorage === "undefined") return;
  const known = read();
  known[kind] = named;
  localStorage.setItem(NAMES, JSON.stringify(known));
}

function read(): Record<string, Name> {
  const written = localStorage.getItem(NAMES);
  if (written === null) return {};
  try {
    const parsed: unknown = JSON.parse(written);
    return typeof parsed === "object" && parsed !== null ? { ...parsed } : {};
  } catch {
    return {};
  }
}
