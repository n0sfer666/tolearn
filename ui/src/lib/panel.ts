const SHUT = "no";

export function shown(key: string): boolean {
  if (typeof localStorage === "undefined") return true;
  return localStorage.getItem(key) !== SHUT;
}

export function remember(key: string, open: boolean): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(key, open ? "yes" : SHUT);
}
