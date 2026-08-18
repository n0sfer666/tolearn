const SHUT = "no";

export function shown(key: string, byDefault = true): boolean {
  if (typeof localStorage === "undefined") return byDefault;
  const kept = localStorage.getItem(key);
  if (kept === null) return byDefault;
  return kept !== SHUT;
}

export function remember(key: string, open: boolean): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(key, open ? "yes" : SHUT);
}
